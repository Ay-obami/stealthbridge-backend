//! Asset-scoped exact value arithmetic. This is domain infrastructure, not
//! asset registry validation or proof of available settlement liquidity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Asset {
    pub network: &'static str,
    pub identity: String,
    pub decimals: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetAmount {
    pub asset: Asset,
    pub minor_units: i128,
}
#[derive(Debug, PartialEq, Eq)]
pub enum AmountError { InvalidAsset, InvalidInput, PrecisionLoss, Overflow, AssetMismatch, InsufficientAmount }

impl Asset {
    pub fn validate(&self) -> Result<(),AmountError> {
        if self.network!="testnet" || self.identity.is_empty() ||
           self.identity.len()>128 || !self.identity.bytes().all(|b|
             b.is_ascii_alphanumeric() || b":_-".contains(&b)) ||
           self.decimals>38 {return Err(AmountError::InvalidAsset);}
        Ok(())
    }
}
impl AssetAmount {
    pub fn parse(asset:Asset, input:&str)->Result<Self,AmountError>{
        asset.validate()?;
        if input.is_empty() || input.starts_with('+') || input.starts_with('-') ||
           !input.bytes().all(|b| b.is_ascii_digit() || b==b'.') ||
           input.bytes().filter(|b| *b==b'.').count()>1 {
            return Err(AmountError::InvalidInput);
        }
        let (whole,decimal)=input.split_once('.').unwrap_or((input,""));
        if whole.is_empty() || (whole.len()>1 && whole.starts_with('0')) ||
           (input.contains('.') && decimal.is_empty()) {
           return Err(AmountError::InvalidInput);
        }
        if decimal.len()>asset.decimals as usize {return Err(AmountError::PrecisionLoss);}
        let pow=10_i128.checked_pow(asset.decimals).ok_or(AmountError::Overflow)?;
        let integer=whole.parse::<i128>().map_err(|_|AmountError::Overflow)?;
        let value=integer.checked_mul(pow).ok_or(AmountError::Overflow)?;
        let fractional=if decimal.is_empty(){0}else{
            let digits=decimal.parse::<i128>().map_err(|_|AmountError::Overflow)?;
            digits.checked_mul(10_i128.checked_pow(asset.decimals-decimal.len() as u32)
                .ok_or(AmountError::Overflow)?).ok_or(AmountError::Overflow)?
        };
        let minor_units=value.checked_add(fractional).ok_or(AmountError::Overflow)?;
        Ok(Self{asset,minor_units})
    }
    pub fn checked_add(&self,other:&Self)->Result<Self,AmountError>{
        if self.asset!=other.asset{return Err(AmountError::AssetMismatch);}
        Ok(Self{asset:self.asset.clone(),minor_units:self.minor_units
           .checked_add(other.minor_units).ok_or(AmountError::Overflow)?})
    }
    pub fn checked_sub(&self,other:&Self)->Result<Self,AmountError>{
        if self.asset!=other.asset{return Err(AmountError::AssetMismatch);}
        let value=self.minor_units.checked_sub(other.minor_units).ok_or(AmountError::Overflow)?;
        if value<0{return Err(AmountError::InsufficientAmount);}
        Ok(Self{asset:self.asset.clone(),minor_units:value})
    }
    pub fn decimal_string(&self)->String{
        let scale=10_i128.pow(self.asset.decimals);
        let whole=self.minor_units/scale;
        let fraction=self.minor_units%scale;
        if fraction==0{return whole.to_string();}
        let mut part=format!("{:0width$}",fraction,width=self.asset.decimals as usize);
        while part.ends_with('0'){part.pop();}
        format!("{whole}.{part}")
    }
}
#[cfg(test)]
mod tests{
  use super::*;
  fn asset()->Asset{Asset{network:"testnet",identity:"verified:token".into(),decimals:7}}
  #[test]fn exact_math(){
    let a=AssetAmount::parse(asset(),"0.1").unwrap();
    let b=AssetAmount::parse(asset(),"0.2").unwrap();
    assert_eq!(a.checked_add(&b).unwrap().decimal_string(),"0.3");
    assert_eq!(b.checked_sub(&a).unwrap().decimal_string(),"0.1");
    assert_eq!(a.minor_units,1_000_000);
  }
  #[test]fn rejects_invalid_decimals_and_formats(){
    for s in ["-1","+1","01","1,000","1e5","1.","1.23456789"," 1",""]{
      assert!(AssetAmount::parse(asset(),s).is_err(),"{s}");
    }
    assert_eq!(AssetAmount::parse(asset(),"1.23456789").unwrap_err(),AmountError::PrecisionLoss);
    let mut invalid=asset();invalid.decimals=39;
    assert_eq!(AssetAmount::parse(invalid,"1").unwrap_err(),AmountError::InvalidAsset);
  }
  #[test]fn prevents_cross_asset_and_overflow(){
    let a=AssetAmount::parse(asset(),"1").unwrap();
    let mut different=asset();different.identity="another".into();
    let b=AssetAmount::parse(different,"1").unwrap();
    assert_eq!(a.checked_add(&b),Err(AmountError::AssetMismatch));
    assert_eq!(a.checked_sub(&AssetAmount::parse(asset(),"2").unwrap()),
      Err(AmountError::InsufficientAmount));
    assert!(AssetAmount::parse(asset(),"999999999999999999999999999999999999").is_err());
  }
}
