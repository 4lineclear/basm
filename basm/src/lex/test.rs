use expect_test::{expect, Expect};

use super::{Advance, BaseLexer, Lexeme, Lexer, RecordedLexer};

fn check(src: &str, expect: Expect) {
    let mut lexer = BaseLexer::new(src);
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
            	0:(7, 22)=Str
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
            	0:(7, 22)=Str
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
            0:(0, 43)=Eol(true)
            43:(43, 44)=Eol(false)
            	44:(44, 51)=Ident
            	44:(51, 52)=Whitespace
            	44:(52, 55)=Ident
            	44:(55, 56)=Whitespace
            	44:(56, 70)=Str
            	44:(70, 71)=Comma
            	44:(71, 72)=Whitespace
            	44:(72, 74)=Digit(Decimal)
            	44:(74, 76)=Whitespace
            44:(76, 106)=Eol(true)
            	106:(106, 112)=Ident
            	106:(112, 114)=Whitespace
            	106:(114, 120)=Ident
            106:(120, 121)=Eol(false)
            121:(121, 122)=Eol(false)
            	122:(122, 128)=Ident
            	122:(128, 129)=Colon
            	122:(129, 130)=Whitespace
            122:(130, 131)=Eol(false)
            	131:(131, 135)=Whitespace
            	131:(135, 138)=Ident
            	131:(138, 145)=Whitespace
            	131:(145, 148)=Ident
            	131:(148, 149)=Comma
            	131:(149, 150)=Whitespace
            	131:(150, 151)=Digit(Decimal)
            	131:(151, 163)=Whitespace
            131:(163, 187)=Eol(true)
            	187:(187, 191)=Whitespace
            	187:(191, 194)=Ident
            	187:(194, 201)=Whitespace
            	187:(201, 204)=Ident
            	187:(204, 205)=Comma
            	187:(205, 206)=Whitespace
            	187:(206, 207)=Digit(Decimal)
            	187:(207, 219)=Whitespace
            187:(219, 245)=Eol(true)
            	245:(245, 249)=Whitespace
            	245:(249, 252)=Ident
            	245:(252, 259)=Whitespace
            	245:(259, 262)=Ident
            	245:(262, 263)=Comma
            	245:(263, 264)=Whitespace
            	245:(264, 271)=Ident
            	245:(271, 277)=Whitespace
            245:(277, 307)=Eol(true)
            	307:(307, 311)=Whitespace
            	307:(311, 314)=Ident
            	307:(314, 321)=Whitespace
            	307:(321, 324)=Ident
            	307:(324, 325)=Comma
            	307:(325, 326)=Whitespace
            	307:(326, 328)=Digit(Decimal)
            	307:(328, 339)=Whitespace
            307:(339, 357)=Eol(true)
            	357:(357, 361)=Whitespace
            	357:(361, 368)=Ident
            	357:(368, 389)=Whitespace
            357:(389, 431)=Eol(true)
            	431:(431, 435)=Whitespace
            	431:(435, 438)=Ident
            	431:(438, 445)=Whitespace
            	431:(445, 448)=Ident
            	431:(448, 449)=Comma
            	431:(449, 450)=Whitespace
            	431:(450, 452)=Digit(Decimal)
            	431:(452, 463)=Whitespace
            431:(463, 486)=Eol(true)
            	486:(486, 490)=Whitespace
            	486:(490, 493)=Ident
            	486:(493, 500)=Whitespace
            	486:(500, 503)=Ident
            	486:(503, 504)=Comma
            	486:(504, 505)=Whitespace
            	486:(505, 508)=Ident
            	486:(508, 518)=Whitespace
            486:(518, 532)=Eol(true)
            	532:(532, 536)=Whitespace
            	532:(536, 543)=Ident
            	532:(543, 564)=Whitespace
            532:(564, 598)=Eol(true)"#]],
    );
}
#[test]
fn print_any() {
    check(
        include_str!("../../../test-sample/3-print-any.asm"),
        expect![[r#"
            0:(0, 0)=Start
            	0:(0, 4)=Whitespace
            	0:(4, 15)=Ident
            	0:(15, 16)=Whitespace
            	0:(16, 19)=Ident
            	0:(19, 20)=Whitespace
            	0:(20, 35)=Str
            	0:(35, 36)=Comma
            	0:(36, 37)=Whitespace
            	0:(37, 39)=Digit(Decimal)
            	0:(39, 40)=Comma
            	0:(40, 41)=Whitespace
            	0:(41, 42)=Digit(Decimal)
            0:(42, 43)=Eol(false)
            	43:(43, 47)=Whitespace
            	43:(47, 55)=Ident
            	43:(55, 56)=Whitespace
            	43:(56, 59)=Ident
            	43:(59, 60)=Whitespace
            	43:(60, 71)=Str
            	43:(71, 72)=Comma
            	43:(72, 73)=Whitespace
            	43:(73, 75)=Digit(Decimal)
            	43:(75, 76)=Comma
            	43:(76, 77)=Whitespace
            	43:(77, 78)=Digit(Decimal)
            43:(78, 79)=Eol(false)
            	79:(79, 83)=Whitespace
            	79:(83, 92)=Ident
            	79:(92, 93)=Whitespace
            	79:(93, 96)=Ident
            	79:(96, 97)=Whitespace
            	79:(97, 129)=Str
            	79:(129, 130)=Comma
            	79:(130, 131)=Whitespace
            	79:(131, 133)=Digit(Decimal)
            	79:(133, 134)=Comma
            	79:(134, 135)=Whitespace
            	79:(135, 136)=Digit(Decimal)
            79:(136, 137)=Eol(false)
            137:(137, 138)=Eol(false)
            	138:(138, 142)=Whitespace
            	138:(142, 148)=Ident
            	138:(148, 149)=Whitespace
            	138:(149, 155)=Ident
            138:(155, 156)=Eol(false)
            156:(156, 157)=Eol(false)
            	157:(157, 163)=Ident
            	157:(163, 164)=Colon
            157:(164, 165)=Eol(false)
            	165:(165, 169)=Whitespace
            	165:(169, 172)=Ident
            	165:(172, 173)=Whitespace
            	165:(173, 176)=Ident
            	165:(176, 177)=Comma
            	165:(177, 178)=Whitespace
            	165:(178, 189)=Ident
            165:(189, 190)=Eol(false)
            	190:(190, 194)=Whitespace
            	190:(194, 198)=Ident
            	190:(198, 199)=Whitespace
            	190:(199, 204)=Ident
            190:(204, 205)=Eol(false)
            205:(205, 206)=Eol(false)
            	206:(206, 210)=Whitespace
            	206:(210, 213)=Ident
            	206:(213, 214)=Whitespace
            	206:(214, 217)=Ident
            	206:(217, 218)=Comma
            	206:(218, 219)=Whitespace
            	206:(219, 227)=Ident
            206:(227, 228)=Eol(false)
            	228:(228, 232)=Whitespace
            	228:(232, 236)=Ident
            	228:(236, 237)=Whitespace
            	228:(237, 242)=Ident
            228:(242, 243)=Eol(false)
            243:(243, 244)=Eol(false)
            	244:(244, 248)=Whitespace
            	244:(248, 251)=Ident
            	244:(251, 252)=Whitespace
            	244:(252, 255)=Ident
            	244:(255, 256)=Comma
            	244:(256, 257)=Whitespace
            	244:(257, 266)=Ident
            244:(266, 267)=Eol(false)
            	267:(267, 271)=Whitespace
            	267:(271, 275)=Ident
            	267:(275, 276)=Whitespace
            	267:(276, 281)=Ident
            267:(281, 282)=Eol(false)
            282:(282, 283)=Eol(false)
            	283:(283, 287)=Whitespace
            	283:(287, 290)=Ident
            	283:(290, 291)=Whitespace
            	283:(291, 294)=Ident
            	283:(294, 295)=Comma
            	283:(295, 296)=Whitespace
            	283:(296, 298)=Digit(Decimal)
            283:(298, 299)=Eol(false)
            	299:(299, 303)=Whitespace
            	299:(303, 306)=Ident
            	299:(306, 307)=Whitespace
            	299:(307, 310)=Ident
            	299:(310, 311)=Comma
            	299:(311, 312)=Whitespace
            	299:(312, 313)=Digit(Decimal)
            299:(313, 314)=Eol(false)
            	314:(314, 318)=Whitespace
            	314:(318, 325)=Ident
            314:(325, 326)=Eol(false)
            	326:(326, 331)=Ident
            	326:(331, 332)=Colon
            326:(332, 333)=Eol(false)
            	333:(333, 337)=Whitespace
            	333:(337, 341)=Ident
            	333:(341, 342)=Whitespace
            	333:(342, 345)=Ident
            333:(345, 346)=Eol(false)
            	346:(346, 350)=Whitespace
            	346:(350, 353)=Ident
            	346:(353, 354)=Whitespace
            	346:(354, 357)=Ident
            	346:(357, 358)=Comma
            	346:(358, 359)=Whitespace
            	346:(359, 360)=Digit(Decimal)
            346:(360, 361)=Eol(false)
            	361:(361, 371)=Ident
            	361:(371, 372)=Colon
            361:(372, 373)=Eol(false)
            	373:(373, 377)=Whitespace
            	373:(377, 380)=Ident
            	373:(380, 381)=Whitespace
            	373:(381, 384)=Ident
            373:(384, 385)=Eol(false)
            	385:(385, 389)=Whitespace
            	385:(389, 392)=Ident
            	385:(392, 393)=Whitespace
            	385:(393, 396)=Ident
            385:(396, 397)=Eol(false)
            	397:(397, 401)=Whitespace
            	397:(401, 404)=Ident
            	397:(404, 405)=Whitespace
            	397:(405, 407)=Ident
            	397:(407, 408)=Comma
            	397:(408, 409)=Whitespace
            	397:(409, 410)=OpenBracket
            	397:(410, 413)=Ident
            	397:(413, 414)=CloseBracket
            397:(414, 415)=Eol(false)
            	415:(415, 419)=Whitespace
            	415:(419, 422)=Ident
            	415:(422, 423)=Whitespace
            	415:(423, 425)=Ident
            	415:(425, 426)=Comma
            	415:(426, 427)=Whitespace
            	415:(427, 428)=Digit(Decimal)
            415:(428, 429)=Eol(false)
            	429:(429, 433)=Whitespace
            	429:(433, 436)=Ident
            	429:(436, 437)=Whitespace
            	429:(437, 447)=Ident
            429:(447, 448)=Eol(false)
            448:(448, 449)=Eol(false)
            	449:(449, 453)=Whitespace
            	449:(453, 456)=Ident
            	449:(456, 457)=Whitespace
            	449:(457, 460)=Ident
            	449:(460, 461)=Comma
            	449:(461, 462)=Whitespace
            	449:(462, 463)=Digit(Decimal)
            449:(463, 464)=Eol(false)
            	464:(464, 468)=Whitespace
            	464:(468, 471)=Ident
            	464:(471, 472)=Whitespace
            	464:(472, 475)=Ident
            	464:(475, 476)=Comma
            	464:(476, 477)=Whitespace
            	464:(477, 478)=Digit(Decimal)
            464:(478, 479)=Eol(false)
            	479:(479, 483)=Whitespace
            	479:(483, 486)=Ident
            	479:(486, 487)=Whitespace
            	479:(487, 490)=Ident
            479:(490, 491)=Eol(false)
            	491:(491, 495)=Whitespace
            	491:(495, 498)=Ident
            	491:(498, 499)=Whitespace
            	491:(499, 502)=Ident
            	491:(502, 503)=Comma
            	491:(503, 504)=Whitespace
            	491:(504, 507)=Ident
            491:(507, 508)=Eol(false)
            	508:(508, 512)=Whitespace
            	508:(512, 519)=Ident
            508:(519, 520)=Eol(false)
            520:(520, 521)=Eol(false)
            	521:(521, 525)=Whitespace
            	521:(525, 528)=Ident
            521:(528, 529)=Eol(false)"#]],
    );
}

#[test]
fn print_int() {
    check(
        include_str!("../../../test-sample/4-print-int.asm"),
        expect![[r#"
            0:(0, 0)=Start
            	0:(0, 4)=Whitespace
            	0:(4, 16)=Ident
            	0:(16, 24)=Whitespace
            	0:(24, 28)=Ident
            	0:(28, 29)=Whitespace
            	0:(29, 32)=Digit(Decimal)
            	0:(32, 36)=Whitespace
            0:(36, 59)=Eol(true)
            	59:(59, 63)=Whitespace
            	59:(63, 79)=Ident
            	59:(79, 83)=Whitespace
            	59:(83, 87)=Ident
            	59:(87, 88)=Whitespace
            	59:(88, 89)=Digit(Decimal)
            	59:(89, 95)=Whitespace
            59:(95, 115)=Eol(true)
            	115:(115, 151)=Whitespace
            115:(151, 194)=Eol(true)
            194:(194, 195)=Eol(false)
            195:(195, 196)=Eol(false)
            	196:(196, 200)=Whitespace
            	196:(200, 206)=Ident
            	196:(206, 207)=Whitespace
            	196:(207, 213)=Ident
            196:(213, 214)=Eol(false)
            214:(214, 215)=Eol(false)
            	215:(215, 221)=Ident
            	215:(221, 222)=Colon
            215:(222, 223)=Eol(false)
            	223:(223, 227)=Whitespace
            	223:(227, 230)=Ident
            	223:(230, 231)=Whitespace
            	223:(231, 234)=Ident
            	223:(234, 235)=Comma
            	223:(235, 236)=Whitespace
            	223:(236, 240)=Digit(Decimal)
            223:(240, 241)=Eol(false)
            	241:(241, 245)=Whitespace
            	241:(245, 249)=Ident
            	241:(249, 250)=Whitespace
            	241:(250, 255)=Ident
            241:(255, 256)=Eol(false)
            256:(256, 257)=Eol(false)
            	257:(257, 261)=Whitespace
            	257:(261, 264)=Ident
            	257:(264, 265)=Whitespace
            	257:(265, 268)=Ident
            	257:(268, 269)=Comma
            	257:(269, 270)=Whitespace
            	257:(270, 272)=Digit(Decimal)
            257:(272, 273)=Eol(false)
            	273:(273, 277)=Whitespace
            	273:(277, 280)=Ident
            	273:(280, 281)=Whitespace
            	273:(281, 284)=Ident
            	273:(284, 285)=Comma
            	273:(285, 286)=Whitespace
            	273:(286, 287)=Digit(Decimal)
            273:(287, 288)=Eol(false)
            	288:(288, 292)=Whitespace
            	288:(292, 299)=Ident
            288:(299, 300)=Eol(false)
            300:(300, 301)=Eol(false)
            	301:(301, 306)=Ident
            	301:(306, 307)=Colon
            301:(307, 308)=Eol(false)
            	308:(308, 312)=Whitespace
            	308:(312, 315)=Ident
            	308:(315, 316)=Whitespace
            	308:(316, 319)=Ident
            	308:(319, 320)=Comma
            	308:(320, 321)=Whitespace
            	308:(321, 333)=Ident
            308:(333, 334)=Eol(false)
            	334:(334, 338)=Whitespace
            	334:(338, 341)=Ident
            	334:(341, 342)=Whitespace
            	334:(342, 345)=Ident
            	334:(345, 346)=Comma
            	334:(346, 347)=Whitespace
            	334:(347, 349)=Digit(Decimal)
            334:(349, 350)=Eol(false)
            	350:(350, 354)=Whitespace
            	350:(354, 357)=Ident
            	350:(357, 358)=Whitespace
            	350:(358, 359)=OpenBracket
            	350:(359, 362)=Ident
            	350:(362, 363)=CloseBracket
            	350:(363, 364)=Comma
            	350:(364, 365)=Whitespace
            	350:(365, 368)=Ident
            350:(368, 369)=Eol(false)
            	369:(369, 373)=Whitespace
            	369:(373, 376)=Ident
            	369:(376, 377)=Whitespace
            	369:(377, 380)=Ident
            369:(380, 381)=Eol(false)
            	381:(381, 385)=Whitespace
            	381:(385, 388)=Ident
            	381:(388, 389)=Whitespace
            	381:(389, 390)=OpenBracket
            	381:(390, 406)=Ident
            	381:(406, 407)=CloseBracket
            	381:(407, 408)=Comma
            	381:(408, 409)=Whitespace
            	381:(409, 412)=Ident
            381:(412, 413)=Eol(false)
            	413:(413, 423)=Ident
            	413:(423, 424)=Colon
            413:(424, 425)=Eol(false)
            	425:(425, 429)=Whitespace
            	425:(429, 432)=Ident
            	425:(432, 433)=Whitespace
            	425:(433, 436)=Ident
            	425:(436, 437)=Comma
            	425:(437, 438)=Whitespace
            	425:(438, 439)=Digit(Decimal)
            425:(439, 440)=Eol(false)
            	440:(440, 444)=Whitespace
            	440:(444, 447)=Ident
            	440:(447, 448)=Whitespace
            	440:(448, 451)=Ident
            	440:(451, 452)=Comma
            	440:(452, 453)=Whitespace
            	440:(453, 455)=Digit(Decimal)
            440:(455, 456)=Eol(false)
            	456:(456, 460)=Whitespace
            	456:(460, 463)=Ident
            	456:(463, 464)=Whitespace
            	456:(464, 467)=Ident
            456:(467, 468)=Eol(false)
            	468:(468, 472)=Whitespace
            	468:(472, 476)=Ident
            	468:(476, 477)=Whitespace
            	468:(477, 480)=Ident
            468:(480, 481)=Eol(false)
            	481:(481, 485)=Whitespace
            	481:(485, 488)=Ident
            	481:(488, 489)=Whitespace
            	481:(489, 492)=Ident
            	481:(492, 493)=Comma
            	481:(493, 494)=Whitespace
            	481:(494, 496)=Digit(Decimal)
            	481:(496, 497)=Whitespace
            481:(497, 541)=Eol(true)
            541:(541, 542)=Eol(false)
            	542:(542, 546)=Whitespace
            	542:(546, 549)=Ident
            	542:(549, 550)=Whitespace
            	542:(550, 553)=Ident
            	542:(553, 554)=Comma
            	542:(554, 555)=Whitespace
            	542:(555, 556)=OpenBracket
            	542:(556, 572)=Ident
            	542:(572, 573)=CloseBracket
            542:(573, 574)=Eol(false)
            	574:(574, 578)=Whitespace
            	574:(578, 581)=Ident
            	574:(581, 582)=Whitespace
            	574:(582, 583)=OpenBracket
            	574:(583, 586)=Ident
            	574:(586, 587)=CloseBracket
            	574:(587, 588)=Comma
            	574:(588, 589)=Whitespace
            	574:(589, 591)=Ident
            574:(591, 592)=Eol(false)
            	592:(592, 596)=Whitespace
            	592:(596, 599)=Ident
            	592:(599, 600)=Whitespace
            	592:(600, 603)=Ident
            592:(603, 604)=Eol(false)
            	604:(604, 608)=Whitespace
            	604:(608, 611)=Ident
            	604:(611, 612)=Whitespace
            	604:(612, 613)=OpenBracket
            	604:(613, 629)=Ident
            	604:(629, 630)=CloseBracket
            	604:(630, 631)=Comma
            	604:(631, 632)=Whitespace
            	604:(632, 635)=Ident
            604:(635, 636)=Eol(false)
            636:(636, 637)=Eol(false)
            	637:(637, 641)=Whitespace
            	637:(641, 644)=Ident
            	637:(644, 645)=Whitespace
            	637:(645, 648)=Ident
            637:(648, 649)=Eol(false)
            	649:(649, 653)=Whitespace
            	649:(653, 656)=Ident
            	649:(656, 657)=Whitespace
            	649:(657, 660)=Ident
            	649:(660, 661)=Comma
            	649:(661, 662)=Whitespace
            	649:(662, 663)=Digit(Decimal)
            649:(663, 664)=Eol(false)
            	664:(664, 668)=Whitespace
            	664:(668, 671)=Ident
            	664:(671, 672)=Whitespace
            	664:(672, 682)=Ident
            664:(682, 683)=Eol(false)
            	683:(683, 693)=Ident
            	683:(693, 694)=Colon
            683:(694, 695)=Eol(false)
            	695:(695, 699)=Whitespace
            	695:(699, 702)=Ident
            	695:(702, 703)=Whitespace
            	695:(703, 706)=Ident
            	695:(706, 707)=Comma
            	695:(707, 708)=Whitespace
            	695:(708, 709)=OpenBracket
            	695:(709, 725)=Ident
            	695:(725, 726)=CloseBracket
            695:(726, 727)=Eol(false)
            727:(727, 728)=Eol(false)
            	728:(728, 732)=Whitespace
            	728:(732, 735)=Ident
            	728:(735, 736)=Whitespace
            	728:(736, 739)=Ident
            	728:(739, 740)=Comma
            	728:(740, 741)=Whitespace
            	728:(741, 742)=Digit(Decimal)
            728:(742, 743)=Eol(false)
            	743:(743, 747)=Whitespace
            	743:(747, 750)=Ident
            	743:(750, 751)=Whitespace
            	743:(751, 754)=Ident
            	743:(754, 755)=Comma
            	743:(755, 756)=Whitespace
            	743:(756, 757)=Digit(Decimal)
            743:(757, 758)=Eol(false)
            	758:(758, 762)=Whitespace
            	758:(762, 765)=Ident
            	758:(765, 766)=Whitespace
            	758:(766, 769)=Ident
            	758:(769, 770)=Comma
            	758:(770, 771)=Whitespace
            	758:(771, 774)=Ident
            758:(774, 775)=Eol(false)
            	775:(775, 779)=Whitespace
            	775:(779, 782)=Ident
            	775:(782, 783)=Whitespace
            	775:(783, 786)=Ident
            	775:(786, 787)=Comma
            	775:(787, 788)=Whitespace
            	775:(788, 789)=Digit(Decimal)
            775:(789, 790)=Eol(false)
            	790:(790, 794)=Whitespace
            	790:(794, 801)=Ident
            790:(801, 802)=Eol(false)
            802:(802, 803)=Eol(false)
            	803:(803, 807)=Whitespace
            	803:(807, 810)=Ident
            	803:(810, 811)=Whitespace
            	803:(811, 814)=Ident
            	803:(814, 815)=Comma
            	803:(815, 816)=Whitespace
            	803:(816, 817)=OpenBracket
            	803:(817, 833)=Ident
            	803:(833, 834)=CloseBracket
            803:(834, 835)=Eol(false)
            	835:(835, 839)=Whitespace
            	835:(839, 842)=Ident
            	835:(842, 843)=Whitespace
            	835:(843, 846)=Ident
            835:(846, 847)=Eol(false)
            	847:(847, 851)=Whitespace
            	847:(851, 854)=Ident
            	847:(854, 855)=Whitespace
            	847:(855, 856)=OpenBracket
            	847:(856, 872)=Ident
            	847:(872, 873)=CloseBracket
            	847:(873, 874)=Comma
            	847:(874, 875)=Whitespace
            	847:(875, 878)=Ident
            847:(878, 879)=Eol(false)
            879:(879, 880)=Eol(false)
            	880:(880, 884)=Whitespace
            	880:(884, 887)=Ident
            	880:(887, 888)=Whitespace
            	880:(888, 891)=Ident
            	880:(891, 892)=Comma
            	880:(892, 893)=Whitespace
            	880:(893, 905)=Ident
            880:(905, 906)=Eol(false)
            	906:(906, 910)=Whitespace
            	906:(910, 913)=Ident
            	906:(913, 914)=Whitespace
            	906:(914, 924)=Ident
            906:(924, 925)=Eol(false)
            925:(925, 926)=Eol(false)
            	926:(926, 930)=Whitespace
            	926:(930, 933)=Ident
            926:(933, 934)=Eol(false)"#]],
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
            	28:(36, 41)=Str
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
            	54:(54, 55)=Colon
            	54:(55, 56)=Whitespace
            	54:(56, 57)=Colon
            	54:(57, 58)=Whitespace
            	54:(58, 59)=Colon
            	54:(59, 60)=Whitespace
            54:(60, 68)=Eol(true)"#]],
    );
}

#[test]
fn rand_other() {
    check(
        "\
label: `~]]./\\
var var var `~]]./\\
var var \"var\", 10, 0`~]]./\\
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
            	35:(43, 48)=Str
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
            	63:(63, 64)=Whitespace
            	63:(64, 66)=Other
            	63:(66, 67)=CloseBracket
            	63:(67, 68)=CloseBracket
            	63:(68, 71)=Other
            63:(71, 79)=Eol(true)"#]],
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

#[test]
fn recorded_equivalent() {
    let src = include_str!("../../../test-sample/0-hello.asm");
    let mut l1 = Vec::new();
    let mut l2 = Vec::new();
    let mut b1 = BaseLexer::new(src);
    let mut b2 = BaseLexer::new(src);
    let mut r1 = RecordedLexer::new(src);
    let mut r2 = RecordedLexer::new(src);
    println!("{b2:?}\n{r2:?}");
    loop {
        let ba1 = b1.advance();
        let ra1 = r1.advance();
        let ba2 = b2.peek();
        let ra2 = r2.peek();
        l1.push(ba1);
        l2.push(ba2);
        assert_eq!(ba1, ra1);
        assert_eq!(ba2, ra2);
        assert_eq!(ba1, ra2);
        assert_eq!(ba2, ra2);
        assert_eq!(b2.pop_peek(), r2.pop_peek());
        if Lexeme::Eof == ba1.lex {
            break;
        }
    }
    assert_eq!(l1, l2);
    assert_eq!(r1.store, r2.store);
    assert_eq!(l1, r1.store);
    assert_eq!(l2, r2.store);
}
