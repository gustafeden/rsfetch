#[allow(dead_code)]
pub struct Logo {
    pub art: &'static str,
    pub name: &'static str,
}

pub fn detect() -> Logo {
    #[cfg(target_os = "macos")]
    {
        apple()
    }
    #[cfg(target_os = "linux")]
    {
        detect_linux()
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        generic()
    }
}

pub fn from_file(path: &str) -> Result<String, std::io::Error> {
    let expanded = if path.starts_with('~') {
        if let Ok(home) = std::env::var("HOME") {
            format!("{}{}", home, &path[1..])
        } else {
            path.to_string()
        }
    } else {
        path.to_string()
    };
    std::fs::read_to_string(expanded)
}

pub fn by_name(name: &str) -> Logo {
    match name.to_lowercase().as_str() {
        "apple" | "macos" | "mac" => apple(),
        "linux" | "tux" => tux(),
        "ubuntu" => ubuntu(),
        "arch" => arch(),
        "debian" => debian(),
        "fedora" => fedora(),
        "none" | "off" => Logo { art: "", name: "none" },
        _ => detect(),
    }
}

#[cfg(target_os = "linux")]
fn detect_linux() -> Logo {
    let id = std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|c| {
            c.lines()
                .find(|l| l.starts_with("ID="))
                .map(|l| l.trim_start_matches("ID=").trim_matches('"').to_lowercase())
        })
        .unwrap_or_default();

    match id.as_str() {
        "ubuntu" => ubuntu(),
        "arch" | "archlinux" => arch(),
        "debian" => debian(),
        "fedora" => fedora(),
        _ => tux(),
    }
}

pub fn apple() -> Logo {
    Logo {
        name: "apple",
        art: "                    'c.
                 ,xNMM.
               .OMMMMo
               OMMM0,
     .;loddo:' loolloddol;.
   cKMMMMMMMMMMNWMMMMMMMMMM0:
 .KMMMMMMMMMMMMMMMMMMMMMMMWd.
 XMMMMMMMMMMMMMMMMMMMMMMMX.
;MMMMMMMMMMMMMMMMMMMMMMMM:
:MMMMMMMMMMMMMMMMMMMMMMMM:
.MMMMMMMMMMMMMMMMMMMMMMMMX.
 kMMMMMMMMMMMMMMMMMMMMMMMMWd.
 .XMMMMMMMMMMMMMMMMMMMMMMMMMMk
  .XMMMMMMMMMMMMMMMMMMMMMMMMK.
    kMMMMMMMMMMMMMMMMMMMMMMd
     ;KMMMMMMMWXXWMMMMMMMk.
       .cooc,.    .,coo:.",
    }
}

pub fn tux() -> Logo {
    Logo {
        name: "linux",
        art: "        a8888b.
       d888888b.
       8P\"YP\"Y88
       8|o||o|88
       8'    .88
       8`._.' Y8.
      d/      `8b.
    .dP   .     Y8b.
   d8:'   \"   `::88b.
  d8\"           `Y88b
 :8P     '       :888
  8a.    :      _a88P
._/\"Yaa_ :    .| 88P|
\\    YP\"      `| 8P  `.
/     \\._____.d|    .'
`--..__)888888P`._.'",
    }
}

pub fn ubuntu() -> Logo {
    Logo {
        name: "ubuntu",
        art: "             .-/+oossssoo+/-.
         `:+ssssssssssssssssss+:`
       -+ssssssssssssssssssyyssss+-
     .ossssssssssssssssssdMMMNysssso.
    /ssssssssssshdmmNNmmyNMMMMhssssss/
   +ssssssssshmydMMMMMMMNddddyssssssss+
  /sssssssshNMMMyhhyyyyhmNMMMNhssssssss/
 .ssssssssdMMMNhsssssssssshNMMMdssssssss.
 +sssshhhyNMMNyssssssssssssyNMMMysssssss+
 ossyNMMMNyMMhsssssssssssssshmmmhssssssso
 ossyNMMMNyMMhsssssssssssssshmmmhssssssso
 +sssshhhyNMMNyssssssssssssyNMMMysssssss+
 .ssssssssdMMMNhsssssssssshNMMMdssssssss.
  /sssssssshNMMMyhhyyyyhdNMMMNhssssssss/
   +sssssssssdmydMMMMMMMMddddyssssssss+
    /ssssssssssshdmNNNNmyNMMMMhssssss/
     .ossssssssssssssssssdMMMNysssso.
       -+sssssssssssssssssyyyssss+-
         `:+ssssssssssssssssss+:`
             .-/+oossssoo+/-.",
    }
}

pub fn arch() -> Logo {
    Logo {
        name: "arch",
        art: "                   -`
                  .o+`
                 `ooo/
                `+oooo:
               `+oooooo:
               -+oooooo+:
             `/:-:++oooo+:
            `/++++/+++++++:
           `/++++++++++++++:
          `/+++ooooooooooooo/`
         ./ooosssso++osssssso+`
        .oossssso-````/ossssss+`
       -osssssso.      :ssssssso.
      :osssssss/        osssso+++.
     /ossssssss/        +ssssooo/-
   `/ossssso+/:-        -:/+osssso+-
  `+sso+:-`                 `.-/+oso:
 `++:.                           `-/+/
 .`                                 `/",
    }
}

pub fn debian() -> Logo {
    Logo {
        name: "debian",
        art: "       _,met$$$$$gg.
    ,g$$$$$$$$$$$$$$$P.
  ,g$$P\"     \"\"\"Y$$.\"$.
 ,$$P'              `$$$.
',$$P       ,ggs.     `$$b:
`d$$'     ,$P\"'   .    $$$
 $$P      d$'     ,    $$P
 $$:      $$.   -    ,d$$'
 $$;      Y$b._   _,d$P'
 Y$$.    `.`\"Y$$$$P\"'
 `$$b      \"-.__
  `Y$$
   `Y$$.
     `$$b.
       `Y$$b.
          `\"Y$b._
              `\"\"\"",
    }
}

pub fn fedora() -> Logo {
    Logo {
        name: "fedora",
        art: "             .',;::::;,'.
         .';:cccccccccccc:;,.
      .;cccccccccccccccccccccc;.
    .:cccccccccccccccccccccccccc:.
   ;ccccccccccccc;.:dddl:.;ccccccc;
  :ccccccccccccc;OWMKOOXMWd;ccccccc:
 :ccccccccccccc;KMMc;cc;xMMc;ccccccc:
 ccccccccccccc;MMM.;cc;;WMW;cccccccc;
 ccccccccccccc;MMM.;cc;;WMW;cccccccc;
 :ccccccccccccc;KMMc;cc;xMMc;ccccccc:
  :ccccccccccccc;OWMKOOXMWd;ccccccc:
   ;ccccccccccccc;.:dddl:.;ccccccc;
    .:cccccccccccccccccccccccccc:.
      .;cccccccccccccccccccccc;.
         .';:cccccccccccc:;,.
             .',;::::;,'.",
    }
}

#[allow(dead_code)]
pub fn generic() -> Logo {
    Logo {
        name: "generic",
        art: "   ___  ___
  |   \\/   |
  |        |
  |  /\\/\\  |
  |_/    \\_|",
    }
}

#[allow(dead_code)]
pub fn available() -> &'static [&'static str] {
    &["apple", "linux", "ubuntu", "arch", "debian", "fedora", "none"]
}
