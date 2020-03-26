use assert_cmd::Command;

#[test]
fn cin_shift_op() -> std::io::Result<()> {
    // basic read is OK
    Command::cargo_bin("shift_op")
        .unwrap()
        .write_stdin("1\n")
        .assert()
        .success();

    // reads the right number
    Command::cargo_bin("shift_op")
        .unwrap()
        .write_stdin("2\n")
        .assert()
        .failure();

    // doesn't read char-by-char
    Command::cargo_bin("shift_op")
        .unwrap()
        .write_stdin("1GARBAGE\n")
        .assert()
        .failure();
    Ok(())
}
