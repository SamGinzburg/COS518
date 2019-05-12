# -*- coding: utf-8 -*-
import matplotlib
import matplotlib.pyplot as plt
import sys
import re

# needed for mu
matplotlib.rc('font', family='Arial')


# arg 1 is the filename for 10 conns
# arg 2 is the filename for 50 conns
# arg 3 is the filename for 100 conns
# arg 4 is the filename for 150 conns
# arg 5 is the filename for 200 conns
# etc...

def get_times(arg, intended_size):
    temp = []
    with open(sys.argv[arg], 'r') as content_file:
        content = content_file.read()
        #print content
        regex = re.compile("\d+\\nROUND TIME ELAPSED \(ms\):\ \d+", re.MULTILINE)
        split = re.findall(regex, content)
        for val in split:
            t = [int(s) for s in val.split() if s.isdigit()]
            # filter out data from trace that isn't from the client
            if t[0] != 0:
                temp.append(t[1])
    print len(temp)
    return [sum(temp) / len(temp)]


mu_100_10 = get_times(1, 10)
mu_100_50 = get_times(2, 50)
mu_100_100 = get_times(3, 100)
mu_100_150 = get_times(4, 150)
mu_100_200 = get_times(5, 200)

mu_200_10 = get_times(6, 10)
mu_200_50 = get_times(7, 50)
mu_200_100 = get_times(8, 100)
mu_200_150 = get_times(9, 150)
mu_200_200 = get_times(10, 200)

mu_300_10 = get_times(11, 10)
mu_300_50 = get_times(12, 50)
mu_300_100 = get_times(13, 100)
mu_300_150 = get_times(14, 150)
mu_300_200 = get_times(15, 200)

# mu = 100
tmp1 = plt.plot([10, 50, 100, 150, 200], [mu_100_10, mu_100_50,
                                          mu_100_100, mu_100_150, mu_100_200],
                                          label = u'μ = 100')
"""
tmp1 = plt.scatter([10] * len(mu_100_10), mu_100_10)
plt.scatter([50] * len(mu_100_50), mu_100_50)
plt.scatter([100] * len(mu_100_100), mu_100_100)
plt.scatter([150] * len(mu_100_150), mu_100_150)
plt.scatter([200] * len(mu_100_200), mu_100_200)
"""
#mu = 200
tmp2 = plt.plot([10, 50, 100, 150, 200], [mu_200_10, mu_200_50,
                                          mu_200_100, mu_200_150, mu_200_200],
                                          label = u'μ = 200')
"""
tmp2 = plt.scatter([10] * len(mu_200_10), mu_200_10, color='black')
plt.scatter([50] * len(mu_200_50), mu_200_50, color='black')
plt.scatter([100] * len(mu_200_100), mu_200_100, color='black')
plt.scatter([150] * len(mu_200_150), mu_200_150, color='black')
plt.scatter([200] * len(mu_200_200), mu_200_200, color='black')
"""
#mu = 300
tmp3 = plt.plot([10, 50, 100, 150, 200], [mu_300_10, mu_300_50,
                                          mu_300_100, mu_300_150, mu_300_200],
                                          label = u'μ = 300')
"""
tmp3 = plt.scatter([10] * len(mu_300_10), mu_300_10, color='red')
plt.scatter([50] * len(mu_300_50), mu_300_50, color='red')
plt.scatter([100] * len(mu_300_100), mu_300_100, color='red')
plt.scatter([150] * len(mu_300_150), mu_300_150, color='red')
plt.scatter([200] * len(mu_300_200), mu_300_200, color='red')
"""

plt.legend(loc='upper left')
plt.xlim([0, 220])
plt.xlabel('Number of Users')
plt.ylabel('Latency (ms)')
#plt.show()
plt.savefig('scaling.png')
