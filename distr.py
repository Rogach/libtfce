from matplotlib import pyplot as pp
from scipy import stats
import numpy as np

xs=np.linspace(0, 10, 100)
for a in np.linspace(0.1,3,15):
    pp.plot(xs, list(map(lambda x: stats.lognorm.pdf(x, a), xs)))
pp.show()
