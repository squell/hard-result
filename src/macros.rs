#[macro_export]
macro_rules! harder {
    (if $cond: tt $body: block else $else: block) => {
        $cond.map(|()| $body).unwrap_or_else(|()| $else)
    };

    (if $cond: tt $body: block) => {
        $cond.map(|()| $body);
    };

    (while $cond: tt $body: block) => {
        // this `.into()` is a known weak spot
        HardBool::r#if(($cond).into(), || {
            HardBool::r#do_while(|| {
                {
                    $body
                }
                {
                    ($cond).into()
                }
            })
        });
    };
}
