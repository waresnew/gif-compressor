alias Rgb=vec3<u32>;

struct GlobalInfo {
    num_frames:u32,
    height:u32,
    width:u32
};

//WARNING:if changing the value or semantics of workgroup sizes, also update the dispatch_workgroups args
const WORKGROUP_SIZE_X=64u;
const WORKGROUP_SIZE_Y=1u;
const WORKGROUP_SIZE_Z=1u;

@group(0) @binding(0) var<uniform> global_info:GlobalInfo;
@group(0) @binding(1) var<storage,read> palettes:array<Rgb>;
@group(0) @binding(2) var<storage,read> palette_offsets:array<u32>;
@group(0) @binding(3) var<storage,read> input_frames:array<Rgb>; //row major
@group(0) @binding(4) var<storage,read_write> output_frames:array<Rgb>; //row major

@compute
@workgroup_size(WORKGROUP_SIZE_X,WORKGROUP_SIZE_Y,WORKGROUP_SIZE_Z)
fn nn_in_palette(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let col_index=global_invocation_id.x;
    let row_index=global_invocation_id.y;
    let frame_index=global_invocation_id.z;
    if frame_index>=global_info.num_frames || row_index>=global_info.height || col_index>=global_info.width {
        return;
    }
    var best_dis=1000000u;
    var ans=Rgb(0,0,0);
    let index=index(frame_index,row_index,col_index);
    for (var i=palette_offsets[frame_index]; i<palette_offsets[frame_index+1];i++) {
        let other=palettes[i];
        let dis=distance_sq(input_frames[index],other);
        if dis<best_dis {
            best_dis=dis;
            ans=other;
        }
    }
    output_frames[index]=ans;

}

@compute
@workgroup_size(WORKGROUP_SIZE_X,WORKGROUP_SIZE_Y,WORKGROUP_SIZE_Z)
fn undither_frame(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let col_index=global_invocation_id.x;
    let row_index=global_invocation_id.y;
    let frame_index=global_invocation_id.z;
    if frame_index>=global_info.num_frames || row_index>=global_info.height || col_index>=global_info.width {
        return;
    }
    var local_input:array<array<Rgb,3>,3>;
    for (var dr=-1;dr<=1;dr++) {
        for (var dc= -1; dc<=1;dc++) {
            let nr=u32(clamp(i32(row_index)+dr,0,i32(global_info.height-1)));
            let nc=u32(clamp(i32(col_index)+dc,0,i32(global_info.width-1)));
            local_input[dr+1][dc+1]=input_frames[index(frame_index,nr,nc)];
        }
    }
    
    let centre=local_input[1][1];
    var luma:array<array<u32,3>,3>;
    for (var i=0;i<3;i++) {
        for (var j=0;j<3;j++) {
            luma[i][j]=rgb_as_luma(local_input[i][j]);
        }
    }
    let prewitt=prewitt_3x3_mag(luma);
    let prewitt_high_threshold = 256u;
    let prewitt_low_threshold = 160u;
    var weight_len=0u;
    var sum_r=0u;
    var sum_g=0u;
    var sum_b=0u;
    var centre_weight:u32;
    if prewitt > prewitt_high_threshold {
        output_frames[index(frame_index,row_index,col_index)]=centre;
        return;
    } else if prewitt > prewitt_low_threshold {
        centre_weight=24;
    } else {
        centre_weight=8;
    }
    weight_len += centre_weight;

    sum_r+=centre_weight*centre.r;
    sum_g+=centre_weight*centre.g;
    sum_b+=centre_weight*centre.b;

    for (var i=0;i<3;i++) {
        for (var j=0;j<3;j++) {
        if i==1 && j==1 {
            continue;
        }
        let neighbour=local_input[i][j];
        let avg=rgb_avg(centre,neighbour);
        let nearest=nn_in_palette_exclude_2(avg,centre,neighbour, frame_index);
        let dis_normalized = f32(distance_sq(avg,nearest))/f32(distance_sq(centre,avg));
        var weight:u32;
        if dis_normalized >= 2 {
            weight=8;
        } else if dis_normalized >= 1 {
            weight=6;
        } else if dis_normalized>= 2.0/3.0 {
            weight=1;
        } else {
            weight=0;
        }
        sum_r += weight * neighbour.r;
        sum_g += weight * neighbour.g;
        sum_b += weight * neighbour.b;
        weight_len += weight;
    }
    }
    output_frames[index(frame_index,row_index,col_index)]= Rgb(sum_r/weight_len,sum_g/weight_len,sum_b/weight_len);
}

fn index(frame:u32, pixel_i:u32, pixel_j:u32)->u32 {
    return frame*global_info.height*global_info.width+pixel_i*global_info.width+pixel_j;
}

//PERF:pretty sure this is the bottleneck
fn nn_in_palette_exclude_2(input:Rgb, exclude1:Rgb,exclude2:Rgb, frame_index:u32)->Rgb {
    var best_dis=1000000u;
    var ans=Rgb(0,0,0);
    for (var i=palette_offsets[frame_index]; i<palette_offsets[frame_index+1];i++) {
        let other=palettes[i];
        if all(other==exclude1)||all(other==exclude2) {
            continue;
        }
        let dis=distance_sq(input,other);
        if dis<best_dis {
            best_dis=dis;
            ans=other;
        }
    }
    return ans;

}
fn rgb_avg(cur:Rgb,other:Rgb)->Rgb {
    return Rgb((cur.r+other.r)/2,(cur.g+other.g)/2,(cur.b+other.b)/2);
}
fn rgb_as_luma(input:Rgb)->u32 {
    return u32(0.299 * f32(input.r) + 0.587 * f32(input.g) + 0.114 * f32(input.b));
}
fn prewitt_3x3_mag(input:array<array<u32,3>,3>)->u32 {

    let gx = i32(input[0][0] + input[1][0] + input[2][0]) - i32(input[0][2]) - i32(input[1][2]) - i32(input[2][2]);
    let gy = i32(input[0][0] + input[0][1] + input[0][2]) - i32(input[2][0]) - i32(input[2][1]) - i32(input[2][2]);
    return u32(sqrt(f32(gx * gx) + f32(gy * gy)));
}
fn distance_sq(cur:Rgb,other:Rgb)->u32  {
    let dr=i32(cur.r)-i32(other.r);
    let dg=i32(cur.g)-i32(other.g);
    let db=i32(cur.b)-i32(other.b);
    return u32(dr*dr+dg*dg+db*db);
}
