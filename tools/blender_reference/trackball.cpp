// SPDX-FileCopyrightText: 2008 Blender Authors
// SPDX-License-Identifier: GPL-2.0-or-later
// calctrackballvec below is unchanged from Blender commit
// 608ad2b3d6bbca0f5d4938048b21744b36d4ebf5.
// Minimal host helpers and fixture emitter added by LokiCraft.
#include <cmath>
#include <algorithm>
#include <iostream>
#include <iomanip>
struct rcti { int xmin,xmax,ymin,ymax; };
int BLI_rcti_size_x(const rcti *r){return r->xmax-r->xmin;}
int BLI_rcti_size_y(const rcti *r){return r->ymax-r->ymin;}
int BLI_rcti_cent_x(const rcti *r){return (r->xmin+r->xmax)/2;}
int BLI_rcti_cent_y(const rcti *r){return (r->ymin+r->ymax)/2;}
float min_ff(float a,float b){return std::min(a,b);}
float len_v2(const float *v){return std::sqrt(v[0]*v[0]+v[1]*v[1]);}
float square_f(float x){return x*x;}
#define V3D_OP_TRACKBALLSIZE 1.1f
void calctrackballvec(const rcti *rect, const int event_xy[2], float r_dir[3])
{
  const float radius = V3D_OP_TRACKBALLSIZE;
  const float t = radius / float(M_SQRT2);
  const float size[2] = {float(BLI_rcti_size_x(rect)), float(BLI_rcti_size_y(rect))};
  /* Aspect correct so dragging in a non-square view doesn't squash the direction.
   * So diagonal motion rotates the same direction the cursor is moving. */
  const float size_min = min_ff(size[0], size[1]);
  const float aspect[2] = {size_min / size[0], size_min / size[1]};

  /* Normalize x and y. */
  r_dir[0] = (event_xy[0] - BLI_rcti_cent_x(rect)) / ((size[0] * aspect[0]) / 2.0);
  r_dir[1] = (event_xy[1] - BLI_rcti_cent_y(rect)) / ((size[1] * aspect[1]) / 2.0);
  const float d = len_v2(r_dir);
  if (d < t) {
    /* Inside sphere. */
    r_dir[2] = sqrtf(square_f(radius) - square_f(d));
  }
  else {
    /* On hyperbola. */
    r_dir[2] = square_f(t) / d;
  }
}

int main(){
 std::cout<<std::setprecision(10)<<"[";
 const int cases[][4]={{800,600,400,300},{800,600,450,320},{800,600,800,600},{1600,600,850,320},{600,1600,350,820},{800,600,-50,100},{100,100,50,100},{100,100,90,50}};
 bool first=true;
 for(auto &c:cases){rcti rect{0,c[0],0,c[1]};int xy[]={c[2],c[3]};float v[3];calctrackballvec(&rect,xy,v);
 if(!first)std::cout<<",";first=false;
 std::cout<<"{\"size\":["<<c[0]<<","<<c[1]<<"],\"point\":["<<c[2]<<","<<c[3]<<"],\"expected\":["<<v[0]<<","<<v[1]<<","<<v[2]<<"]}";}
 std::cout<<"]\n";
}
