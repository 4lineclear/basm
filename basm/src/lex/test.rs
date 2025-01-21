use expect_test::{expect, Expect};

use crate::lex::Lexeme;

use super::{Advance, Lexer};

fn check(src: &str, expect: Expect) {
    let mut lexer = Lexer::new(src);
    let output = "0:(0, 0)=Start".to_owned()
        + &std::iter::from_fn(|| match lexer.advance() {
            Advance {
                lex: Lexeme::Eof, ..
            } => None,
            Advance {
                lex, offset, span, ..
            } => {
                let p = matches!(lex, Lexeme::Eol(_)).then_some("").unwrap_or("\t");
                Some(format!("\n{p}{offset}:{span:?}={lex:?}"))
            }
        })
        .collect::<String>();

    expect.assert_eq(&output);
}

#[test]
fn empty() {
    check("", expect!["0:(0, 0)=Start"]);
}
#[test]
fn multi_empty() {
    check(
        "\n\n\n\n",
        expect![[r#"
            0:(0, 0)=Start
            0:(0, 1)=Eol(false)
            1:(1, 2)=Eol(false)
            2:(2, 3)=Eol(false)
            3:(3, 4)=Eol(false)"#]],
    );
}
#[test]
fn empty_ws() {
    check(
        "\t\t    \t     \t ",
        expect![[r#"
            0:(0, 0)=Start
            	0:(0, 14)=Whitespace"#]],
    );
}
#[test]
fn multi_empty_ws() {
    check(
        "\t\t    \t     \t \n\n\t\t    \t     \t \n\n",
        expect![[r#"
            0:(0, 0)=Start
            	0:(0, 14)=Whitespace
            0:(14, 15)=Eol(false)
            15:(15, 16)=Eol(false)
            	16:(16, 30)=Whitespace
            16:(30, 31)=Eol(false)
            31:(31, 32)=Eol(false)"#]],
    );
}
#[test]
fn singles() {
    check(
        "one\n two\t\n threeeeee\n four\n",
        expect![[r#"
            0:(0, 0)=Start
            	0:(0, 3)=Ident
            0:(3, 4)=Eol(false)
            	4:(4, 5)=Whitespace
            	4:(5, 8)=Ident
            	4:(8, 9)=Whitespace
            4:(9, 10)=Eol(false)
            	10:(10, 11)=Whitespace
            	10:(11, 20)=Ident
            10:(20, 21)=Eol(false)
            	21:(21, 22)=Whitespace
            	21:(22, 26)=Ident
            21:(26, 27)=Eol(false)"#]],
    );
}
#[test]
fn doubles() {
    check(
        "one two\n two threeeee\t\n threeeeee four\n four five\n",
        expect![[r#"
            0:(0, 0)=Start
            	0:(0, 3)=Ident
            	0:(3, 4)=Whitespace
            	0:(4, 7)=Ident
            0:(7, 8)=Eol(false)
            	8:(8, 9)=Whitespace
            	8:(9, 12)=Ident
            	8:(12, 13)=Whitespace
            	8:(13, 21)=Ident
            	8:(21, 22)=Whitespace
            8:(22, 23)=Eol(false)
            	23:(23, 24)=Whitespace
            	23:(24, 33)=Ident
            	23:(33, 34)=Whitespace
            	23:(34, 38)=Ident
            23:(38, 39)=Eol(false)
            	39:(39, 40)=Whitespace
            	39:(40, 44)=Ident
            	39:(44, 45)=Whitespace
            	39:(45, 49)=Ident
            39:(49, 50)=Eol(false)"#]],
    );
}
#[test]
fn triples() {
    check(
        "one two, three\n two threeeee, four",
        expect![[r#"
            0:(0, 0)=Start
            	0:(0, 3)=Ident
            	0:(3, 4)=Whitespace
            	0:(4, 7)=Ident
            	0:(7, 8)=Comma
            	0:(8, 9)=Whitespace
            	0:(9, 14)=Ident
            0:(14, 15)=Eol(false)
            	15:(15, 16)=Whitespace
            	15:(16, 19)=Ident
            	15:(19, 20)=Whitespace
            	15:(20, 28)=Ident
            	15:(28, 29)=Comma
            	15:(29, 30)=Whitespace
            	15:(30, 34)=Ident"#]],
    );
}
#[test]
fn empty_comments() {
    check(
        "\
\t\t  \t    ; one two three
; three four five
\t; five six",
        expect![[r#"
            0:(0, 0)=Start
            	0:(0, 9)=Whitespace
            0:(9, 25)=Eol(true)
            25:(25, 43)=Eol(true)
            	43:(43, 44)=Whitespace
            43:(44, 55)=Eol(true)"#]],
    );
}
#[test]
fn comment_etc() {
    check(
        "\
abc\t\t  \t    ; one two three
cde efg; three four five
ghi ijk, klm\t; five six",
        expect![[r#"
            0:(0, 0)=Start
            	0:(0, 3)=Ident
            	0:(3, 12)=Whitespace
            0:(12, 28)=Eol(true)
            	28:(28, 31)=Ident
            	28:(31, 32)=Whitespace
            	28:(32, 35)=Ident
            28:(35, 53)=Eol(true)
            	53:(53, 56)=Ident
            	53:(56, 57)=Whitespace
            	53:(57, 60)=Ident
            	53:(60, 61)=Comma
            	53:(61, 62)=Whitespace
            	53:(62, 65)=Ident
            	53:(65, 66)=Whitespace
            53:(66, 77)=Eol(true)"#]],
    );
}
#[test]
fn label() {
    check(
        "\
l1      :
\t\tl2\t:
\t\t    l3      :",
        expect![[r#"
            0:(0, 0)=Start
            	0:(0, 2)=Ident
            	0:(2, 8)=Whitespace
            	0:(8, 9)=Colon
            0:(9, 10)=Eol(false)
            	10:(10, 12)=Whitespace
            	10:(12, 14)=Ident
            	10:(14, 15)=Whitespace
            	10:(15, 16)=Colon
            10:(16, 17)=Eol(false)
            	17:(17, 23)=Whitespace
            	17:(23, 25)=Ident
            	17:(25, 31)=Whitespace
            	17:(31, 32)=Colon"#]],
    );
}
#[test]
fn section() {
    check(
        "\
section data one
section bss
section text",
        expect![[r#"
            0:(0, 0)=Start
            	0:(0, 7)=Ident
            	0:(7, 8)=Whitespace
            	0:(8, 12)=Ident
            	0:(12, 13)=Whitespace
            	0:(13, 16)=Ident
            0:(16, 17)=Eol(false)
            	17:(17, 24)=Ident
            	17:(24, 25)=Whitespace
            	17:(25, 28)=Ident
            17:(28, 29)=Eol(false)
            	29:(29, 36)=Ident
            	29:(36, 37)=Whitespace
            	29:(37, 41)=Ident"#]],
    );
}
#[test]
fn decimal() {
    check(
        "\
1, 1293, 9384093, 1231234",
        expect![[r#"
            0:(0, 0)=Start
            	0:(0, 1)=Digit(Decimal)
            	0:(1, 2)=Comma
            	0:(2, 3)=Whitespace
            	0:(3, 7)=Digit(Decimal)
            	0:(7, 8)=Comma
            	0:(8, 9)=Whitespace
            	0:(9, 16)=Digit(Decimal)
            	0:(16, 17)=Comma
            	0:(17, 18)=Whitespace
            	0:(18, 25)=Digit(Decimal)"#]],
    );
}
#[test]
fn variable() {
    check(
        "\
msg db \"ONE TWO THREE\", 12309, 12
digit reb 100",
        expect![[r#"
            0:(0, 0)=Start
            	0:(0, 3)=Ident
            	0:(3, 4)=Whitespace
            	0:(4, 6)=Ident
            	0:(6, 7)=Whitespace
            	0:(7, 22)=String
            	0:(22, 23)=Comma
            	0:(23, 24)=Whitespace
            	0:(24, 29)=Digit(Decimal)
            	0:(29, 30)=Comma
            	0:(30, 31)=Whitespace
            	0:(31, 33)=Digit(Decimal)
            0:(33, 34)=Eol(false)
            	34:(34, 39)=Ident
            	34:(39, 40)=Whitespace
            	34:(40, 43)=Ident
            	34:(43, 44)=Whitespace
            	34:(44, 47)=Digit(Decimal)"#]],
    );
}
#[test]
fn variable_err() {
    check(
        r#"msg db "ONE TWO THREE" 12309 12"#,
        expect![[r#"
            0:(0, 0)=Start
            	0:(0, 3)=Ident
            	0:(3, 4)=Whitespace
            	0:(4, 6)=Ident
            	0:(6, 7)=Whitespace
            	0:(7, 22)=String
            	0:(22, 23)=Whitespace
            	0:(23, 28)=Digit(Decimal)
            	0:(28, 29)=Whitespace
            	0:(29, 31)=Digit(Decimal)"#]],
    );
}
#[test]
fn hello_world() {
    check(
        include_str!("../../../test-sample/0-hello.asm"),
        expect![[r#"
            0:(0, 0)=Start
            	0:(0, 7)=Ident
            	0:(7, 10)=Whitespace
            	0:(10, 14)=Ident
            0:(14, 15)=Eol(false)
            	15:(15, 19)=Whitespace
            	15:(19, 26)=Ident
            	15:(26, 27)=Whitespace
            	15:(27, 29)=Ident
            	15:(29, 30)=Whitespace
            	15:(30, 44)=String
            	15:(44, 45)=Comma
            	15:(45, 46)=Whitespace
            	15:(46, 48)=Digit(Decimal)
            	15:(48, 50)=Whitespace
            15:(50, 80)=Eol(true)
            80:(80, 81)=Eol(false)
            	81:(81, 88)=Ident
            	81:(88, 91)=Whitespace
            	81:(91, 95)=Ident
            81:(95, 96)=Eol(false)
            	96:(96, 100)=Whitespace
            	96:(100, 106)=Ident
            	96:(106, 110)=Whitespace
            	96:(110, 116)=Ident
            96:(116, 117)=Eol(false)
            117:(117, 118)=Eol(false)
            	118:(118, 124)=Ident
            	118:(124, 125)=Colon
            	118:(125, 126)=Whitespace
            118:(126, 127)=Eol(false)
            	127:(127, 131)=Whitespace
            	127:(131, 134)=Ident
            	127:(134, 141)=Whitespace
            	127:(141, 144)=Ident
            	127:(144, 145)=Comma
            	127:(145, 146)=Whitespace
            	127:(146, 147)=Digit(Decimal)
            	127:(147, 159)=Whitespace
            127:(159, 183)=Eol(true)
            	183:(183, 187)=Whitespace
            	183:(187, 190)=Ident
            	183:(190, 197)=Whitespace
            	183:(197, 200)=Ident
            	183:(200, 201)=Comma
            	183:(201, 202)=Whitespace
            	183:(202, 203)=Digit(Decimal)
            	183:(203, 215)=Whitespace
            183:(215, 241)=Eol(true)
            	241:(241, 245)=Whitespace
            	241:(245, 248)=Ident
            	241:(248, 255)=Whitespace
            	241:(255, 258)=Ident
            	241:(258, 259)=Comma
            	241:(259, 260)=Whitespace
            	241:(260, 267)=Ident
            	241:(267, 273)=Whitespace
            241:(273, 303)=Eol(true)
            	303:(303, 307)=Whitespace
            	303:(307, 310)=Ident
            	303:(310, 317)=Whitespace
            	303:(317, 320)=Ident
            	303:(320, 321)=Comma
            	303:(321, 322)=Whitespace
            	303:(322, 324)=Digit(Decimal)
            	303:(324, 335)=Whitespace
            303:(335, 353)=Eol(true)
            	353:(353, 357)=Whitespace
            	353:(357, 364)=Ident
            	353:(364, 385)=Whitespace
            353:(385, 427)=Eol(true)
            	427:(427, 431)=Whitespace
            	427:(431, 434)=Ident
            	427:(434, 441)=Whitespace
            	427:(441, 444)=Ident
            	427:(444, 445)=Comma
            	427:(445, 446)=Whitespace
            	427:(446, 448)=Digit(Decimal)
            	427:(448, 459)=Whitespace
            427:(459, 482)=Eol(true)
            	482:(482, 486)=Whitespace
            	482:(486, 489)=Ident
            	482:(489, 496)=Whitespace
            	482:(496, 499)=Ident
            	482:(499, 500)=Comma
            	482:(500, 501)=Whitespace
            	482:(501, 504)=Ident
            	482:(504, 514)=Whitespace
            482:(514, 528)=Eol(true)
            	528:(528, 532)=Whitespace
            	528:(532, 539)=Ident
            	528:(539, 560)=Whitespace
            528:(560, 594)=Eol(true)"#]],
    );
}
#[test]
fn print_any() {
    check(
        include_str!("../../../test-sample/3-print-any.asm"),
        expect![[r#"
            0:(0, 0)=Start
            	0:(0, 7)=Ident
            	0:(7, 8)=Whitespace
            	0:(8, 12)=Ident
            0:(12, 13)=Eol(false)
            	13:(13, 17)=Whitespace
            	13:(17, 28)=Ident
            	13:(28, 29)=Whitespace
            	13:(29, 31)=Ident
            	13:(31, 32)=Whitespace
            	13:(32, 47)=String
            	13:(47, 48)=Comma
            	13:(48, 49)=Whitespace
            	13:(49, 51)=Digit(Decimal)
            	13:(51, 52)=Comma
            	13:(52, 53)=Whitespace
            	13:(53, 54)=Digit(Decimal)
            13:(54, 55)=Eol(false)
            	55:(55, 59)=Whitespace
            	55:(59, 67)=Ident
            	55:(67, 68)=Whitespace
            	55:(68, 70)=Ident
            	55:(70, 71)=Whitespace
            	55:(71, 82)=String
            	55:(82, 83)=Comma
            	55:(83, 84)=Whitespace
            	55:(84, 86)=Digit(Decimal)
            	55:(86, 87)=Comma
            	55:(87, 88)=Whitespace
            	55:(88, 89)=Digit(Decimal)
            55:(89, 90)=Eol(false)
            	90:(90, 94)=Whitespace
            	90:(94, 103)=Ident
            	90:(103, 104)=Whitespace
            	90:(104, 106)=Ident
            	90:(106, 107)=Whitespace
            	90:(107, 139)=String
            	90:(139, 140)=Comma
            	90:(140, 141)=Whitespace
            	90:(141, 143)=Digit(Decimal)
            	90:(143, 144)=Comma
            	90:(144, 145)=Whitespace
            	90:(145, 146)=Digit(Decimal)
            90:(146, 147)=Eol(false)
            147:(147, 148)=Eol(false)
            	148:(148, 155)=Ident
            	148:(155, 156)=Whitespace
            	148:(156, 160)=Ident
            148:(160, 161)=Eol(false)
            	161:(161, 165)=Whitespace
            	161:(165, 171)=Ident
            	161:(171, 172)=Whitespace
            	161:(172, 178)=Ident
            161:(178, 179)=Eol(false)
            179:(179, 180)=Eol(false)
            	180:(180, 186)=Ident
            	180:(186, 187)=Colon
            180:(187, 188)=Eol(false)
            	188:(188, 192)=Whitespace
            	188:(192, 195)=Ident
            	188:(195, 196)=Whitespace
            	188:(196, 199)=Ident
            	188:(199, 200)=Comma
            	188:(200, 201)=Whitespace
            	188:(201, 212)=Ident
            188:(212, 213)=Eol(false)
            	213:(213, 217)=Whitespace
            	213:(217, 221)=Ident
            	213:(221, 222)=Whitespace
            	213:(222, 227)=Ident
            213:(227, 228)=Eol(false)
            228:(228, 229)=Eol(false)
            	229:(229, 233)=Whitespace
            	229:(233, 236)=Ident
            	229:(236, 237)=Whitespace
            	229:(237, 240)=Ident
            	229:(240, 241)=Comma
            	229:(241, 242)=Whitespace
            	229:(242, 250)=Ident
            229:(250, 251)=Eol(false)
            	251:(251, 255)=Whitespace
            	251:(255, 259)=Ident
            	251:(259, 260)=Whitespace
            	251:(260, 265)=Ident
            251:(265, 266)=Eol(false)
            266:(266, 267)=Eol(false)
            	267:(267, 271)=Whitespace
            	267:(271, 274)=Ident
            	267:(274, 275)=Whitespace
            	267:(275, 278)=Ident
            	267:(278, 279)=Comma
            	267:(279, 280)=Whitespace
            	267:(280, 289)=Ident
            267:(289, 290)=Eol(false)
            	290:(290, 294)=Whitespace
            	290:(294, 298)=Ident
            	290:(298, 299)=Whitespace
            	290:(299, 304)=Ident
            290:(304, 305)=Eol(false)
            305:(305, 306)=Eol(false)
            	306:(306, 310)=Whitespace
            	306:(310, 313)=Ident
            	306:(313, 314)=Whitespace
            	306:(314, 317)=Ident
            	306:(317, 318)=Comma
            	306:(318, 319)=Whitespace
            	306:(319, 321)=Digit(Decimal)
            306:(321, 322)=Eol(false)
            	322:(322, 326)=Whitespace
            	322:(326, 329)=Ident
            	322:(329, 330)=Whitespace
            	322:(330, 333)=Ident
            	322:(333, 334)=Comma
            	322:(334, 335)=Whitespace
            	322:(335, 336)=Digit(Decimal)
            322:(336, 337)=Eol(false)
            	337:(337, 341)=Whitespace
            	337:(341, 348)=Ident
            337:(348, 349)=Eol(false)
            	349:(349, 354)=Ident
            	349:(354, 355)=Colon
            349:(355, 356)=Eol(false)
            	356:(356, 360)=Whitespace
            	356:(360, 364)=Ident
            	356:(364, 365)=Whitespace
            	356:(365, 368)=Ident
            356:(368, 369)=Eol(false)
            	369:(369, 373)=Whitespace
            	369:(373, 376)=Ident
            	369:(376, 377)=Whitespace
            	369:(377, 380)=Ident
            	369:(380, 381)=Comma
            	369:(381, 382)=Whitespace
            	369:(382, 383)=Digit(Decimal)
            369:(383, 384)=Eol(false)
            	384:(384, 394)=Ident
            	384:(394, 395)=Colon
            384:(395, 396)=Eol(false)
            	396:(396, 400)=Whitespace
            	396:(400, 403)=Ident
            	396:(403, 404)=Whitespace
            	396:(404, 407)=Ident
            396:(407, 408)=Eol(false)
            	408:(408, 412)=Whitespace
            	408:(412, 415)=Ident
            	408:(415, 416)=Whitespace
            	408:(416, 419)=Ident
            408:(419, 420)=Eol(false)
            	420:(420, 424)=Whitespace
            	420:(424, 427)=Ident
            	420:(427, 428)=Whitespace
            	420:(428, 430)=Ident
            	420:(430, 431)=Comma
            	420:(431, 432)=Whitespace
            	420:(432, 433)=OpenBracket
            	420:(433, 436)=Ident
            	420:(436, 437)=CloseBracket
            420:(437, 438)=Eol(false)
            	438:(438, 442)=Whitespace
            	438:(442, 445)=Ident
            	438:(445, 446)=Whitespace
            	438:(446, 448)=Ident
            	438:(448, 449)=Comma
            	438:(449, 450)=Whitespace
            	438:(450, 451)=Digit(Decimal)
            438:(451, 452)=Eol(false)
            	452:(452, 456)=Whitespace
            	452:(456, 459)=Ident
            	452:(459, 460)=Whitespace
            	452:(460, 470)=Ident
            452:(470, 471)=Eol(false)
            471:(471, 472)=Eol(false)
            	472:(472, 476)=Whitespace
            	472:(476, 479)=Ident
            	472:(479, 480)=Whitespace
            	472:(480, 483)=Ident
            	472:(483, 484)=Comma
            	472:(484, 485)=Whitespace
            	472:(485, 486)=Digit(Decimal)
            472:(486, 487)=Eol(false)
            	487:(487, 491)=Whitespace
            	487:(491, 494)=Ident
            	487:(494, 495)=Whitespace
            	487:(495, 498)=Ident
            	487:(498, 499)=Comma
            	487:(499, 500)=Whitespace
            	487:(500, 501)=Digit(Decimal)
            487:(501, 502)=Eol(false)
            	502:(502, 506)=Whitespace
            	502:(506, 509)=Ident
            	502:(509, 510)=Whitespace
            	502:(510, 513)=Ident
            502:(513, 514)=Eol(false)
            	514:(514, 518)=Whitespace
            	514:(518, 521)=Ident
            	514:(521, 522)=Whitespace
            	514:(522, 525)=Ident
            	514:(525, 526)=Comma
            	514:(526, 527)=Whitespace
            	514:(527, 530)=Ident
            514:(530, 531)=Eol(false)
            	531:(531, 535)=Whitespace
            	531:(535, 542)=Ident
            531:(542, 543)=Eol(false)
            543:(543, 544)=Eol(false)
            	544:(544, 548)=Whitespace
            	544:(548, 551)=Ident
            544:(551, 552)=Eol(false)"#]],
    );
}

#[test]
fn numerics() {
    check(
        "\
190238, 10928321, 0904832041, 3924092840238491019283210
0x1saklj90238SLKDJSD, 0b10101_1001, 0o172537162
",
        expect![[r#"
            0:(0, 0)=Start
            	0:(0, 6)=Digit(Decimal)
            	0:(6, 7)=Comma
            	0:(7, 8)=Whitespace
            	0:(8, 16)=Digit(Decimal)
            	0:(16, 17)=Comma
            	0:(17, 18)=Whitespace
            	0:(18, 28)=Digit(Decimal)
            	0:(28, 29)=Comma
            	0:(29, 30)=Whitespace
            	0:(30, 55)=Digit(Decimal)
            0:(55, 56)=Eol(false)
            	56:(56, 59)=Digit(Hex)
            	56:(59, 76)=Ident
            	56:(76, 77)=Comma
            	56:(77, 78)=Whitespace
            	56:(78, 90)=Digit(Binary)
            	56:(90, 91)=Comma
            	56:(91, 92)=Whitespace
            	56:(92, 103)=Digit(Octal)
            56:(103, 104)=Eol(false)"#]],
    );
}

#[test]
fn deref() {
    check(
        "\
one [ yeahhhhh
two [ nooooo 12309]
three [  12309 nooooo]
",
        expect![[r#"
            0:(0, 0)=Start
            	0:(0, 3)=Ident
            	0:(3, 4)=Whitespace
            	0:(4, 5)=OpenBracket
            	0:(5, 6)=Whitespace
            	0:(6, 14)=Ident
            0:(14, 15)=Eol(false)
            	15:(15, 18)=Ident
            	15:(18, 19)=Whitespace
            	15:(19, 20)=OpenBracket
            	15:(20, 21)=Whitespace
            	15:(21, 27)=Ident
            	15:(27, 28)=Whitespace
            	15:(28, 33)=Digit(Decimal)
            	15:(33, 34)=CloseBracket
            15:(34, 35)=Eol(false)
            	35:(35, 40)=Ident
            	35:(40, 41)=Whitespace
            	35:(41, 42)=OpenBracket
            	35:(42, 44)=Whitespace
            	35:(44, 49)=Digit(Decimal)
            	35:(49, 50)=Whitespace
            	35:(50, 56)=Ident
            	35:(56, 57)=CloseBracket
            35:(57, 58)=Eol(false)"#]],
    );
}

#[test]
fn float() {
    check(
        "\
0.123.123.123
",
        expect![[r#"
            0:(0, 0)=Start
            	0:(0, 1)=Digit(Decimal)
            	0:(1, 2)=Other
            	0:(2, 5)=Digit(Decimal)
            	0:(5, 6)=Other
            	0:(6, 9)=Digit(Decimal)
            	0:(9, 10)=Other
            	0:(10, 13)=Digit(Decimal)
            0:(13, 14)=Eol(false)"#]],
    );
}

/// out of order
#[test]
fn ooo() {
    check(
        "\
123  yeah
123, yeah
",
        expect![[r#"
            0:(0, 0)=Start
            	0:(0, 3)=Digit(Decimal)
            	0:(3, 5)=Whitespace
            	0:(5, 9)=Ident
            0:(9, 10)=Eol(false)
            	10:(10, 13)=Digit(Decimal)
            	10:(13, 14)=Comma
            	10:(14, 15)=Whitespace
            	10:(15, 19)=Ident
            10:(19, 20)=Eol(false)"#]],
    );
}

#[test]
fn other_comment() {
    check(
        "\
; one two three
    ; one two three
",
        expect![[r#"
            0:(0, 0)=Start
            0:(0, 16)=Eol(true)
            	16:(16, 20)=Whitespace
            16:(20, 36)=Eol(true)"#]],
    );
}

#[test]
fn colon_other() {
    check(
        "\
label: : :
var var var: : :
var var \"var\", 10, 0: : :
section section: : :
: : : ;empty!
",
        expect![[r#"
            0:(0, 0)=Start
            	0:(0, 5)=Ident
            	0:(5, 6)=Colon
            	0:(6, 7)=Whitespace
            	0:(7, 8)=Colon
            	0:(8, 9)=Whitespace
            	0:(9, 10)=Colon
            0:(10, 11)=Eol(false)
            	11:(11, 14)=Ident
            	11:(14, 15)=Whitespace
            	11:(15, 18)=Ident
            	11:(18, 19)=Whitespace
            	11:(19, 22)=Ident
            	11:(22, 23)=Colon
            	11:(23, 24)=Whitespace
            	11:(24, 25)=Colon
            	11:(25, 26)=Whitespace
            	11:(26, 27)=Colon
            11:(27, 28)=Eol(false)
            	28:(28, 31)=Ident
            	28:(31, 32)=Whitespace
            	28:(32, 35)=Ident
            	28:(35, 36)=Whitespace
            	28:(36, 41)=String
            	28:(41, 42)=Comma
            	28:(42, 43)=Whitespace
            	28:(43, 45)=Digit(Decimal)
            	28:(45, 46)=Comma
            	28:(46, 47)=Whitespace
            	28:(47, 48)=Digit(Decimal)
            	28:(48, 49)=Colon
            	28:(49, 50)=Whitespace
            	28:(50, 51)=Colon
            	28:(51, 52)=Whitespace
            	28:(52, 53)=Colon
            28:(53, 54)=Eol(false)
            	54:(54, 61)=Ident
            	54:(61, 62)=Whitespace
            	54:(62, 69)=Ident
            	54:(69, 70)=Colon
            	54:(70, 71)=Whitespace
            	54:(71, 72)=Colon
            	54:(72, 73)=Whitespace
            	54:(73, 74)=Colon
            54:(74, 75)=Eol(false)
            	75:(75, 76)=Colon
            	75:(76, 77)=Whitespace
            	75:(77, 78)=Colon
            	75:(78, 79)=Whitespace
            	75:(79, 80)=Colon
            	75:(80, 81)=Whitespace
            75:(81, 89)=Eol(true)"#]],
    );
}

#[test]
fn rand_other() {
    check(
        "\
label: `~]]./\\
var var var `~]]./\\
var var \"var\", 10, 0`~]]./\\
section section`~]]./\\
 `~]]./\\;empty!
",
        expect![[r#"
            0:(0, 0)=Start
            	0:(0, 5)=Ident
            	0:(5, 6)=Colon
            	0:(6, 7)=Whitespace
            	0:(7, 9)=Other
            	0:(9, 10)=CloseBracket
            	0:(10, 11)=CloseBracket
            	0:(11, 14)=Other
            0:(14, 15)=Eol(false)
            	15:(15, 18)=Ident
            	15:(18, 19)=Whitespace
            	15:(19, 22)=Ident
            	15:(22, 23)=Whitespace
            	15:(23, 26)=Ident
            	15:(26, 27)=Whitespace
            	15:(27, 29)=Other
            	15:(29, 30)=CloseBracket
            	15:(30, 31)=CloseBracket
            	15:(31, 34)=Other
            15:(34, 35)=Eol(false)
            	35:(35, 38)=Ident
            	35:(38, 39)=Whitespace
            	35:(39, 42)=Ident
            	35:(42, 43)=Whitespace
            	35:(43, 48)=String
            	35:(48, 49)=Comma
            	35:(49, 50)=Whitespace
            	35:(50, 52)=Digit(Decimal)
            	35:(52, 53)=Comma
            	35:(53, 54)=Whitespace
            	35:(54, 55)=Digit(Decimal)
            	35:(55, 57)=Other
            	35:(57, 58)=CloseBracket
            	35:(58, 59)=CloseBracket
            	35:(59, 62)=Other
            35:(62, 63)=Eol(false)
            	63:(63, 70)=Ident
            	63:(70, 71)=Whitespace
            	63:(71, 78)=Ident
            	63:(78, 80)=Other
            	63:(80, 81)=CloseBracket
            	63:(81, 82)=CloseBracket
            	63:(82, 85)=Other
            63:(85, 86)=Eol(false)
            	86:(86, 87)=Whitespace
            	86:(87, 89)=Other
            	86:(89, 90)=CloseBracket
            	86:(90, 91)=CloseBracket
            	86:(91, 94)=Other
            86:(94, 102)=Eol(true)"#]],
    );
}

#[test]
fn empty_not_instruction() {
    check(
        "one, two, three\n two, threeeee, four",
        expect![[r#"
            0:(0, 0)=Start
            	0:(0, 3)=Ident
            	0:(3, 4)=Comma
            	0:(4, 5)=Whitespace
            	0:(5, 8)=Ident
            	0:(8, 9)=Comma
            	0:(9, 10)=Whitespace
            	0:(10, 15)=Ident
            0:(15, 16)=Eol(false)
            	16:(16, 17)=Whitespace
            	16:(17, 20)=Ident
            	16:(20, 21)=Comma
            	16:(21, 22)=Whitespace
            	16:(22, 30)=Ident
            	16:(30, 31)=Comma
            	16:(31, 32)=Whitespace
            	16:(32, 36)=Ident"#]],
    );
}
