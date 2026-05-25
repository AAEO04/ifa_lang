def loop_sum(limit):
    sum = 0
    i = 0
    while i < limit:
        sum = sum + i
        i = i + 1
    return sum

result = loop_sum(10000000)
print(result)
