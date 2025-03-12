private _array = [1,2,3,4];
private _last = _array select (count _array - 1);

private _array2 = [[1,2,3],[4,5,6],[7,8,9]];
private _very_last = (_array2 select 2) select (count (_array2 select 2) - 1);
