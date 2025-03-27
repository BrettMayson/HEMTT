const date = new Date();

if (!String.prototype.padStart) {
    String.prototype.padStart = function padStart(targetLength, padString) {
        targetLength = targetLength >> 0; //floor if number or convert non-number to 0;
        padString = String(padString || ' ');
        if (this.length > targetLength) {
            return String(this);
        }
        else {
            targetLength = targetLength - this.length;
            if (targetLength > padString.length) {
                padString += padString.repeat(targetLength / padString.length); //append to original to ensure we are longer than needed
            }
            return padString.slice(0, targetLength) + String(this);
        }
    };
}

if (!Number.prototype.padStart) {
    Number.prototype.padStart = function padStart(targetLength, padString) {
        return String(this).padStart(targetLength, padString);
    };
}

var ex1date = date.getFullYear() + "-" + (date.getMonth().padStart(2, '0')) + "-" + date.getDate().padStart(2, '0');
var ex1time = date.getHours().padStart(2, '0') + ":" + date.getMinutes().padStart(2, '0') + ":" + date.getSeconds().padStart(2, '0');
var ex1 = ex1date + " " + ex1time;

var ex2 = date.getFullYear().toString().slice(-2) + (date.getMonth().padStart(2, '0')) + date.getDate().padStart(2, '0');

var content = document.getElementById("content");
content.innerHTML = content.innerHTML.replace(/{{ time_1 }}/g, ex1).replace(/{{ time_2 }}/g, ex2);
