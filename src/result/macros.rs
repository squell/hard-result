#[macro_export]
macro_rules! harder {
    (if $cond: tt $body: block else $else: block) => {
        $cond.map(|()| $body).unwrap_or_else(|()| $else)
    };

    (if $cond: tt $body: block) => {
        $cond.map(|()| $body);
    };

    (while $cond: tt $body: block) => {
        HardBool::r#while(move || $cond, move || $body);
    };
}
