# -*- coding: utf-8 -*-
import matplotlib.pyplot as plt
import matplotlib
matplotlib.rc('font', family='Arial')

# hardcode the values from a mu=100k round
total = float(375058)
network = 42028 + 72816 + 33627 + 64318
app = 85 + 88 + 181 + 180 + 283 + 275
sample = 183 + 371 + 551
inverse = 25 + 58 + 81
forward_dec = 0 + 20271 + 39948
forward_noise = 63469 + 30295 + 297
backwards_enc = 135 + 290 + 626
other = total - network - app - inverse - sample - forward_noise - forward_dec - backwards_enc

# Pie chart, where the slices will be ordered and plotted counter-clockwise:
labels = 'Network', 'Apply', 'Inverse', 'Sample', 'Forward Decrypt', 'Forward Noise', 'Backwards Re-Encrypt', 'Other'
sizes = [(network/total) * 100, (app/total) * 100,
         (inverse/total) * 100, (sample/total) * 100, (forward_dec/total) * 100, (forward_noise/total) * 100,
         (backwards_enc/total) * 100, (other/total) * 100]
print sizes
explode = (0, 0, 0, 0, 0, 0, 0, 0)  # only "explode" the 2nd slice (i.e. 'Hogs')

fig1, ax1 = plt.subplots()
ax1.pie(sizes, explode=explode, #autopct='%1.3f%%', turn off percents, unreadable otherwise
        shadow=False, startangle=90)
ax1.axis('equal')  # Equal aspect ratio ensures that pie is drawn as a circle.

circle = plt.Circle(xy=(0,0), radius=0.75, facecolor='white')
plt.gca().add_artist(circle)

new_labels = [x + " " + str(round(y, 2)) + "%" for x, y in zip(labels, sizes)]

plt.legend(labels=new_labels, bbox_to_anchor=(1,0))
plt.title(u'Breakdown of Round Time for μ = 100,000 Across All Servers')
#plt.show()
plt.savefig('breakdown.png', bbox_inches="tight")
