function loop_sum(limit) {
    let sum = 0;
    let i = 0;
    while (i < limit) {
        sum = sum + i;
        i = i + 1;
    }
    return sum;
}

const result = loop_sum(10000000);
console.log(result);
