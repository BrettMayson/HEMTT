// Ignored:
a && b;
a && {call qq}; // dummy call to avoid s26
a && {(call qq) && {(call ww) && {(call ee) && {(call ff)}}}}; // correct
a && (call qq) && (call ww) && (call ee) && (call ff); // not atempting short-cirucitng (see only_code config option)


// Linted:
a && {(call qq)} && {(call ww)} && {(call ee)} && {(call ff)}; // bad chain
!(!a || {!b} || {!c} || {!d} || {!e}); // bad chain
