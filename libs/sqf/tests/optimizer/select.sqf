[1,2,3,4,5] select x;       // ok
something select [2,3];     // ok   
[[-1], [1]] select z;       // ok (with +copy)
