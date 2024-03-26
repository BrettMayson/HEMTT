private _isWater = [0, 1] select (surfaceIsWater getPos player);
private _isLand = [1, 0] select (surfaceIsWater getPos player);
private _isHEMTT = [1, 0] select (name player != "HEMTT");
private _isFuelEmpty = [1,0] select (round fuel vehicle player);
