extern crate fix_rs;

use fix_rs::fix::{parse_message,print_group};

fn main() {
    //let message = "8=FIX.4.2\u{1}9=251\u{1}35=D\u{1}49=AFUNDMGR\u{1}56=ABROKER\u{1}34=2\u{1}52=20030615-01:14:49\u{1}11=12345\u{1}1=111111\u{1}63=0\u{1}64=20030621\u{1}21=3\u{1}110=1000\u{1}111=50000\u{1}55=IBM\u{1}48=459200101\u{1}22=1\u{1}54=1\u{1}60=2003061501:14:49\u{1}38=5000\u{1}40=1\u{1}44=15.75\u{1}15=USD\u{1}59=0\u{1}10=221\u{1}";
    let message = "8=FIX.4.2\u{1}9=65\u{1}35=A\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=062\u{1}";

    let tags = parse_message(message).unwrap();
    print_group(&tags,0);
}
