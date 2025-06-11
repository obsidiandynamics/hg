pub fn unqualified_type_name<T: ?Sized>() -> &'static str {
    let fq_type_name = std::any::type_name::<T>();
    match fq_type_name.rfind("::") {
        None => fq_type_name,
        Some(last_index) => &fq_type_name[last_index + 2..]
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn unqualified_type_name_custom_type() {
        struct Dummy;
        let u_name = super::unqualified_type_name::<Dummy>();
        assert_eq!("Dummy", u_name);
    }
    
    #[test]
    fn unqualified_type_name_intrinsic_type() {
        let u_name = super::unqualified_type_name::<str>();
        assert_eq!("str", u_name);
    }
}