pub fn wrap<S: Into<String>>(source: S) -> String {
    let head = 
r#"
<html>
  <head>
    <title>HEMTT PreProcess Inspection</title>
    <style>
        .code {
            padding: 1rem;
            font: 'Lucida Console';
            font-size: 1.5rem;
            background-color: #1E1E1E;
            color: #D4D4D4;
        }
        .info {
            color: #4EC9B0;
            text-decoration:underline;
            text-decoration-style: double;
        }
        .keyword {
            color: #4FC1FF;
        }
    </style>
    <body>
        <h1>HEMTT PreProcess Inspection</h1>
        <div class="code">
<pre>
"#;
    let foot =
r#"
</pre>
        </div>
    </body>
  </head>
</html>
"#;
    format!("{}{}{}", head, source.into(), foot)
}
