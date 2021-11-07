#!/usr/bin/python
import matplotlib
import matplotlib.pyplot as plt
import matplotlib.animation as anim
plt.style.use('seaborn-whitegrid')
import threading
import numpy as np
import sys
# from numpy import loadtext
import time
import csv

import signal
import os
def signal_handler(sig, frame):
        print('You pressed Ctrl+C!')
        os._exit(1)
signal.signal(signal.SIGINT, signal_handler)



plt.style.use('dark_background')
plt.figure(figsize=(60,60))
# ax.spines['bottom'].set_color('#dddddd')
# ax.spines['top'].set_color('#dddddd')
# ax.spines['right'].set_color('red')
# ax.spines['left'].set_color('red')
# ax.tick_params(axis='x', colors='red')
# ax.tick_params(axis='y', colors='red')
# ax.yaxis.label.set_color('red')
# ax.xaxis.label.set_color('red')
# ax.title.set_color('red')
matplotlib.rc('axes',edgecolor='#333333', labelcolor='#ffffff')
matplotlib.rc('xtick',color='#666666')
matplotlib.rc('ytick',color='#666666')
matplotlib.rc('figure',facecolor='#000000')
matplotlib.rc('lines',color='#666666')
matplotlib.rc('patch',edgecolor='#666666')
matplotlib.rc('grid',color='#333333')
matplotlib.rc('font', family='monospace', size=20)
[i.set_color("#ccccce") for i in plt.gca().get_xticklabels()]
[i.set_color("#ccccce") for i in plt.gca().get_yticklabels()]

for param, value in plt.rcParams.items():
    # if 'color' in param: plt.rcParams[param] = '#00ff00'
    if 'font' in param:
        if '.family' in param: plt.rcParams[param] = ['monospace']
        # print (param, value)


_i = -1
_op = 0.5
_sz = 100
_colors = [
    [1.0,.4,0,_op],
    [.0,.9,.2,_op],
    [.9,.95,0,_op],
    [.4,.6,.9,_op],
    [.3,.1,.8,_op],
    [0.9,.1,.9,_op],
    [.4,.7,.9,_op],
    [.7,0,1,_op],
    [.8,.8,.8,_op],
    [.9,.4,.7,_op],
    [.9,.5,.6,_op],
]



class FileTailer(object):
    def __init__(self, file, delay=0.1):
        self.file = file
        self.delay = delay
    def __iter__(self):
        while True:
            where = self.file.tell()
            line = self.file.readline()
            if line and line.endswith('\n'): # only emit full lines
                yield line
            else:                            # for a partial line, pause and back up
                time.sleep(self.delay)       # ...not actually a recommended approach.
                self.file.seek(where)
def each(iterable, n=1):
    l = len(iterable)
    for ndx in range(0, l, n):
        yield iterable[ndx:min(ndx + n, l)]




def draw(file, x, y):
    global _i
    global _colors, _sz
    _i+= 1
    color = _colors[_i%len(_colors)]
    plt.title(sys.argv[1])
    print '%s\tdraw %d from %d to %d -- max: %f -- color: ' % (file, len(x), x[0], x[-1], max(y)), color
    darken = color[:]
    darken[0]*= .8
    darken[1]*= .8
    darken[2]*= .8
    darken[3]*= 1
    plt.scatter(x, y, color=color, edgecolors=darken, s=_sz)
    # return
    # print len(avg)
    darken[0] = (darken[0] + .5) /1.4
    darken[1] = (darken[1] + .5) /1.4
    darken[2] = (darken[2] + .5) /1.4
    darken[3] = .9

    _x = []
    avgx = []
    minx = []
    maxx = []
    ii = 0
    for i in each(y, 50):
        avg = 0
        _max = -99999
        _min = 99999
        for j in i:
            avg+= j
            if _max < j: _max = j
            if _min > j: _min = j
        avg/= len(i)
        ii+= len(i)
        _x.append(x[ii-1])
        avgx.append(avg)
        maxx.append(_max)
        minx.append(_min)
    plt.plot(_x, maxx, color=darken, linewidth=2)
    plt.plot(_x, avgx, color=darken, linewidth=2)
    plt.plot(_x, minx, color=darken, linewidth=2)


    # maxx = []
    # may = []
    # for i in range(0,len(x)):
    #     if not len(may) or y[i] > may[len(may)-1]:
    #         maxx.append(x[i])
    #         may.append(y[i])
    #
    # plt.plot(maxx, may, color=darken, linewidth=2)
    #
    # N = 1000
    # if len(x) > N:
    #     avg = np.convolve(y, np.ones((N,))/N, mode='valid')
    #     print '--avg-max: %s' % max(avg)
    #     plt.plot(x[N-1:], avg, color=darken, linewidth=2)

last_vx = True
def thread(file):
    global last_vx
    with open(file,'r') as csvfile:
        plots = csv.reader(csvfile, delimiter='\t')

        x = y = None
        minx = None
        offset = 0 if last_vx is True or last_vx is False else last_vx
        # if last_vx and last_vx is not True: minx = last_vx
        i = 0
        for row in plots:
            i+= 1
            row[0] = i
            # print row
            if int(row[0]) == 1 or x is None:
                if x is not None and len(x): draw(x, y)
                x = []
                y = []
            try:
                vx = int(row[0])
                vx = float(row[3])/1000.0/60.0
                if minx is None: minx = vx
                vx-= minx
                if last_vx and vx: last_vx = vx
                # vx+= offset
                #print vx
                # vy = float(row[7]) # gp nodes
                vy = float(row[0]) # exp num
                vy = float(row[2]) # score
                vy = float(row[11]) # NN node count
                vy = float(row[14]) # NN link count
                vy = float(row[1]) # success
                # if vx > 5: break
                # if vy < -1: continue
                # if vy < 0: vy = 0
                # if vy == 0: continue
                # vy = float(row[0]) # num
                # if vx < 14: continue
                # if vx > 5: break
                # if vx < 300: continue
                # if vy < 00: continue
                x.append(vx)
                y.append(vy)
                # plt.show()
            except Exception, e:
                print e
                continue
        if x is not None and len(x): draw(file, x, y)


def main():
    for file in sys.argv[1:]:
        # anim.FuncAnimation(fig, animate, interval=1000)

        # th = threading.Thread(target=thread, args=(file,))
        # th.start()
        thread(file)

    plt.show()
    # while True:
        # if refresh:
            # plt.show()
        #     refresh = False
        # else:
        #     time.sleep(.5)

main()
