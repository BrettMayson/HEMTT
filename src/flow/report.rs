struct Report {
    warnings: Vec<FileMessage>,
    errors: Vec<FileMessage>,
    can_proceed: bool,
}
