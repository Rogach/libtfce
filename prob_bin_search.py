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
    if var <= 0:
        return 0
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
            if z_x_m <= z < z_x_p:
                x += d
                z = z_x_p
                changed = True
            elif z_x_m > z >= z_x_p:
                x -= d
                z = z_x_m
                changed = True
            else:
                on_x = False

        on_y = True
        while on_y:
            z_y_m = fn(x, y - d)
            z_y_p = fn(x, y + d)
            if z_y_m <= z < z_y_p:
                y += d
                z = z_y_p
                changed = True
            elif z_y_m > z >= z_y_p:
                y -= d
                z = z_y_m
                changed = True
            else:
                on_y = False

        if not changed:
            d /= 2

    return (x, y)


def compute_posterior(tests, approx):

    greater_tests = list(map(lambda t: t[0], filter(lambda t: t[1] > 0, tests)))
    less_tests = list(map(lambda t: t[0], filter(lambda t: t[1] < 0, tests)))
    if len(greater_tests) > 0 and len(less_tests) > 0:
        fuzzy_upper_bound = np.mean(greater_tests)
        fuzzy_lower_bound = np.mean(less_tests)
    else:
        fuzzy_upper_bound = 2
        fuzzy_lower_bound = -2

    m = (fuzzy_lower_bound + fuzzy_upper_bound) / 2
    v = max(abs(fuzzy_upper_bound - fuzzy_lower_bound) / 4, 0.3)
    d = 0.1

    fn = lambda m, v: probability_of_evidence(tests, m, v)
    max_mean, max_var = hill_climb(m, v, d, 0.001, fn)

    # grid = []
    # print(max_mean, max_var)
    # print(fn(max_mean, max_var))
    # print(fn(max_mean, max_var*2))
    # for x in np.linspace(6, 10, 100):
    #     d = []
    #     for y in np.linspace(0.001, 0.5, 100):
    #         d.append(probability_of_evidence(tests, x, y))
    #     grid.append(d)
    # pp.imshow(grid, cmap="coolwarm", extent=(0.001, 0.5, 10, 6), aspect=0.12)
    # pp.show()

    if approx:
        # (var, total_p) = integrate_1d(max_var/100, max_var*10, max_var, lambda v: fn(max_mean, v), 0)
        # return N(max_mean, var / total_p)
        (mean, var, total_p) = integrate_2d(m - 10, m + 10, max_var/100, max_var*10, max_mean, max_var, fn, 0)
        return N(mean / total_p, var / total_p)
    else:
        return N(max_mean, max_var)

def integrate_1d(x_min, x_max, cx, fn, l):
    v1 = fn(x_min)
    v2 = fn(x_max)
    lx = (cx - x_min) / (x_max - x_min)
    expected = v1 * lx + v2 * (1 - lx)
    real = fn(cx)
    if abs(real - expected) < 0.01:
        P = (v1 + v2) / 2 * (x_max - x_min)
        X = (x_max - x_min) / 6 * (x_min*(2*v1 + v2) + x_max*(v1 + 2*v2))
        return (X, P)
    else:
        (x1, p1) = integrate_1d(x_min, cx, (x_min+cx)/2, fn, l+1)
        (x2, p2) = integrate_1d(cx, x_max, (cx+x_max)/2, fn, l+1)
        return (x1+x2, p1+p2)

def integrate_2d(x_min, x_max, y_min, y_max, cx, cy, fn, l):
    #print("%sintegrate_2d(x_min=%s, x_max=%s, y_min=%s, y_max=%s, cx=%s, cy=%s)" % ("  " * l, x_min, x_max, y_min, y_max, cx, cy))
    v1 = fn(x_min, y_min)
    v2 = fn(x_max, y_min)
    v3 = fn(x_max, y_max)
    v4 = fn(x_min, y_max)

    lx = (cx - x_min) / (x_max - x_min)
    ly = (cy - y_min) / (y_max - y_min)
    expected = v1*lx*ly + v2*(1-lx)*ly + v3*(1-lx)*(1-ly) + v4*lx*(1-ly)
    vc = fn(cx, cy)
    #print("%s expected %s, got %s, diff %s" % ("  " * l, expected, vc, abs(vc - expected)))
    if abs(vc - expected) < 0.05:
        XY = (x_max - x_min) * (y_max - y_min)
        P = XY * (v1 + v2 + v3 + v4)/4
        X = XY / 12 * (x_min*(2*(v1+v4) + v2 + v3) + x_max*(2*(v2+v3) + v1 + v4))
        Y = XY / 12 * (y_min*(2*(v1+v2) + v3 + v4) + y_max*(2*(v3+v4) + v1 + v2))
        return (X, Y, P)
    else:
        (x1, y1, p1) = integrate_2d(x_min, cx, y_min, cy, (x_min+cx)/2, (y_min+cy)/2, fn, l+1)
        (x2, y2, p2) = integrate_2d(cx, x_max, y_min, cy, (cx+x_max)/2, (y_min+cy)/2, fn, l+1)
        (x3, y3, p3) = integrate_2d(x_min, cx, cy, y_max, (x_min+cx)/2, (cy+y_max)/2, fn, l+1)
        (x4, y4, p4) = integrate_2d(cx, x_max, cy, y_max, (cx+x_max)/2, (cy+y_max)/2, fn, l+1)
        return (x1+x2+x3+x4, y1+y2+y3+y4, p1+p2+p3+p4)

def do_test(prior, target_mean, target_var):
    t = stats.norm.rvs(prior.mean, prior.var+1)
    r = stats.norm.rvs(target_mean, target_var)
    if t < r:
        return (t, +1)
    else:
        return (t, -1)

mean_history_s = []
var_history_s = []

mean_history_approx_s = []
var_history_approx_s = []

for n in range(1, 10):
    prior = N(5, 5)
    prior_approx = N(5, 5)

    target_mean = 8
    target_var = 0.5

    tests = []
    tests_approx = []

    mean_history = []
    var_history = []
    mean_history_approx = []
    var_history_approx = []

    for x in range(1, 200):
        tests.append(do_test(prior, target_mean, target_var))
        posterior = compute_posterior(tests, False)
        mean_history.append(posterior.mean)
        var_history.append(posterior.var)
        prior = posterior

        tests_approx.append(do_test(prior_approx, target_mean, target_var))
        posterior_approx = compute_posterior(tests_approx, False)
        mean_history_approx.append(posterior_approx.mean)
        var_history_approx.append(posterior_approx.var)
        prior_approx = posterior_approx

        print(n, x, prior, prior_approx)
    mean_history_s.append(mean_history)
    var_history_s.append(var_history)
    mean_history_approx_s.append(mean_history_approx)
    var_history_approx_s.append(var_history_approx)

pp.subplot(2, 1, 1)
pp.title("mean")
for mean_history in mean_history_s:
    pp.plot(mean_history, 'b')
for mean_history_approx in mean_history_approx_s:
    pp.plot(mean_history_approx, 'g')
pp.axhline(y=target_mean, color="r")

pp.subplot(2, 1, 2)
pp.title("var")
for var_history in var_history_s:
    pp.plot(var_history, 'b')
for var_history_approx in var_history_approx_s:
    pp.plot(var_history_approx, 'g')
pp.axhline(y=target_var, color="r")

pp.show()
