private _myVar = 10;
private _myFunc={
	params["_param1","_param2"];
	private _result = _param1+_param2;_result
};
private _result =if(([1,2] call _myFunc)==3)then{
"Equal to 3"
} else { "Not equal to 3"};
systemChat _result;
