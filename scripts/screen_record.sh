#!/bin/bash
ffmpeg -f x11grab -framerate 30 -i :0.0+0,0 -filter_complex "settb=1/1000,setpts=RTCTIME/1000-1500000000000,mpdecimate,split[out][ts];[out]setpts=N/FRAME_RATE/TB[out]" -map "[out]" -vcodec libx264 -pix_fmt yuv420p -preset fast -crf 0 -threads 0 nodups.mkv -map "[ts]" -f mkvtimestamp_v2 nodups.txt -vsync 0
