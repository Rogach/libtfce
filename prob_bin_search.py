import numpy as np
from matplotlib import pyplot as pp
from mpl_toolkits.mplot3d import Axes3D
from matplotlib import cm
from scipy import stats
import math

def norm_pdf(x, mean, variance):
    return 1/(math.sqrt(2 * math.pi * variance**2)) * math.e**(-((x-mean)**2)/(2*variance**2))

def norm_cdf(x, mean, variance):
    b0 = 0.2316419
    b1 = 0.319381530
    b2 = -0.356563782
    b3 = 1.781477937
    b4 = -1.821255978
    b5 = 1.330274429
    x -= mean
    x /= variance
    if x > 0:
        t = 1 / (1 + b0 * x)
        ts = (b1*t + b2*(t**2) + b3*(t**3) + b4*(t**4) + b5*(t**5))
        return 1 - norm_pdf(x, 0, 1) * ts
    else:
        x = -x
        t = 1 / (1 + b0 * x)
        ts = (b1*t + b2*(t**2) + b3*(t**3) + b4*(t**4) + b5*(t**5))
        return norm_pdf(x, 0, 1) * ts


class N:
    def __init__(self, mean, variance):
        self.mean = mean;
        self.var = variance;
    def __str__(self):
        return "N(mean=" + str(self.mean) + ", variance=" + str(self.var) + ")"
    def pdf(self, x):
        return norm_pdf(x, self.mean, self.var)

def probability_of_evidence(tests, mean, var):
    pt = 1
    for t in tests:
        if t[1] > 0:
            pt *= (1 - norm_cdf(t[0], mean, var))
        else:
            pt *= norm_cdf(t[0], mean, var)
    return pt

def hill_climb(sx, sy, sd, min_d, fn):
    x = sx
    y = sy
    z = fn(x, y)
    d = sd

    while d >= min_d:
        changed = False
        on_x = True
        while on_x:
            z_x_m = fn(x - d, y)
            z_x_p = fn(x + d, y)
            if z_x_m < z < z_x_p:
                x += d
                z = z_x_p
                changed = True
            elif z_x_m > z > z_x_p:
                x -= d
                z = z_x_m
                changed = True
            else:
                on_x = False

        on_y = True
        while on_y:
            z_y_m = fn(x, y - d)
            z_y_p = fn(x, y + d)
            if z_y_m < z < z_y_p:
                y += d
                z = z_y_p
                changed = True
            elif z_y_m > z > z_y_p:
                y -= d
                z = z_y_m
                changed = True
            else:
                on_y = False

        if not changed:
            d /= 2

    return (x, y)


def compute_posterior(tests):

    greater_tests = list(map(lambda t: t[0], filter(lambda t: t[1] > 0, tests)))
    less_tests = list(map(lambda t: t[0], filter(lambda t: t[1] < 0, tests)))
    if len(greater_tests) > 0 and len(less_tests) > 0:
        fuzzy_upper_bound = np.mean(greater_tests)
        fuzzy_lower_bound = np.mean(less_tests)
    else:
        fuzzy_upper_bound = 2
        fuzzy_lower_bound = -2

    m = (fuzzy_lower_bound + fuzzy_upper_bound) / 2
    v = abs(fuzzy_upper_bound - fuzzy_lower_bound) / 4
    d = 0.1

    max_mean, max_var = hill_climb(m, v, d, 0.001, lambda m, v: probability_of_evidence(tests, m, v))
    return N(max_mean, max_var)

def do_test(prior, target_mean, target_var):
    t = stats.norm.ppf(np.random.uniform(0, 1), prior.mean, prior.var)
    r = stats.norm.rvs(target_mean, target_var)
    if t < r:
        return (t, +1)
    else:
        return (t, -1)

prior = N(5, 5)

target_mean = 8
target_var = 0.5

tests = []

mean_history = []
var_history = []

for x in range(1, 300):
    tests.append(do_test(prior, target_mean, target_var))
    posterior = compute_posterior(tests)
    print(posterior)
    mean_history.append(posterior.mean)
    var_history.append(posterior.var)
    prior = posterior


pp.subplot(2, 1, 1)
pp.title("mean")
pp.plot(mean_history, 'b')
pp.axhline(y=target_mean, color="r")

pp.subplot(2, 1, 2)
pp.title("var")
pp.plot(var_history, 'b')
pp.axhline(y=target_var, color="r")

pp.show()
