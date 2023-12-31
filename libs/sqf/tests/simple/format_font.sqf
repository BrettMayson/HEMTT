#define QUOTE(var1) #var1

#define FORMAT_TEXT_GREEN QUOTE(<font color='#00FF00'>%1</font>)
#define FORMAT_TEXT_RED QUOTE(<font color='#FF0000'>%1</font>)

private _redTextDisabled = format[(FORMAT_TEXT_RED), "thing"];
private _greenTextEnabled = format[(FORMAT_TEXT_GREEN), "thing"];
