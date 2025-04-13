use rk_cad::CadModel;

/// 内部の CadModel（Block 型）から、FreeCAD で読み込める STEP ファイル文字列を生成する。
///
/// ※この実装は、Block 型（原点と寸法を持つ直交の立方体）を対象に、あらかじめ決めたテンプレート形式を用いて出力する例です。
pub fn write_step(model: &CadModel) -> String {
    // 今回は簡単のため、CadModel に Block が 1 つだけ存在する前提とする
    let block = &model.blocks[0];
    let ox = block.origin.x;
    let oy = block.origin.y;
    let oz = block.origin.z;
    let dx = block.dimensions.x;
    let dy = block.dimensions.y;
    let dz = block.dimensions.z;
    let ox_plus_dx = ox + dx;
    let oy_plus_dy = oy + dy;
    let oz_plus_dz = oz + dz;
    // 固定のタイムスタンプ（必要なら動的に生成してください）
    let timestamp = "2025-04-14T15:30:00";

    // 以下は、FreeCAD が出力した立方体 STEP ファイル（再掲例）の必要最低限部分に近いテンプレート例です。
    // ※ 改行や空白、エンティティ番号はテンプレートの内容に合わせています。
    // 本来、STEP ファイル生成はエンティティ間の参照解決などが必要ですが、今回は単一立方体の出力例として
    // テンプレートに対する置換処理で実現しています。
    let step_str = format!(r#"ISO-10303-21;
HEADER;
FILE_DESCRIPTION(('FreeCAD Minimal Cube'),'2;1');
FILE_NAME('Cube.step','{timestamp}',(''),(''),'Open CASCADE STEP processor','FreeCAD','Unknown');
FILE_SCHEMA(('AUTOMOTIVE_DESIGN {{ 1 0 10303 214 1 1 1 1 }}'));
ENDSEC;
DATA;
#1 = APPLICATION_PROTOCOL_DEFINITION('international standard','automotive_design',2000,#2);
#2 = APPLICATION_CONTEXT('core data for automotive mechanical design processes');
#3 = SHAPE_DEFINITION_REPRESENTATION(#4,#10);
#4 = PRODUCT_DEFINITION_SHAPE('','',#5);
#5 = PRODUCT_DEFINITION('design','',#6,#9);
#6 = PRODUCT_DEFINITION_FORMATION('','',#7);
#7 = PRODUCT('{name}','{name}','',(#8));
#8 = PRODUCT_CONTEXT('',#2,'mechanical');
#9 = PRODUCT_DEFINITION_CONTEXT('part definition',#2,'design');
#10 = ADVANCED_BREP_SHAPE_REPRESENTATION('',(#11,#15),#165);
#11 = AXIS2_PLACEMENT_3D('',#12,#13,#14);
#12 = CARTESIAN_POINT('',({ox:.1},{oy:.1},{oz:.1}));
#13 = DIRECTION('',(0.,0.,1.));
#14 = DIRECTION('',(1.,0.,-0.));
#15 = MANIFOLD_SOLID_BREP('',#16);
#16 = CLOSED_SHELL('',(#17,#57,#97,#119,#141,#153));
#17 = ADVANCED_FACE('',(#18),#52,.F.);
#18 = FACE_BOUND('',#19,.F.);
#19 = EDGE_LOOP('',(#20,#30,#38,#46));
#20 = ORIENTED_EDGE('',*,*,#21,.F.);
#21 = EDGE_CURVE('',#22,#24,#26,.T.);
#22 = VERTEX_POINT('',#23);
#23 = CARTESIAN_POINT('',({ox:.1},{oy:.1},{oz:.1}));
#24 = VERTEX_POINT('',#25);
#25 = CARTESIAN_POINT('',({ox:.1},{oy:.1},{oz_plus_dz:.1}));
#26 = LINE('',#27,#28);
#27 = CARTESIAN_POINT('',({ox:.1},{oy:.1},{oz:.1}));
#28 = VECTOR('',#29,1.);
#29 = DIRECTION('',(0.,0.,1.));
#30 = ORIENTED_EDGE('',*,*,#31,.T.);
#31 = EDGE_CURVE('',#22,#32,#34,.T.);
#32 = VERTEX_POINT('',#33);
#33 = CARTESIAN_POINT('',({ox:.1},{oy_plus_dy:.1},{oz:.1}));
#34 = LINE('',#35,#36);
#35 = CARTESIAN_POINT('',({ox:.1},{oy:.1},{oz:.1}));
#36 = VECTOR('',#37,1.);
#37 = DIRECTION('',(-0.,1.,0.));
#38 = ORIENTED_EDGE('',*,*,#39,.T.);
#39 = EDGE_CURVE('',#32,#40,#42,.T.);
#40 = VERTEX_POINT('',#41);
#41 = CARTESIAN_POINT('',({ox:.1},{oy_plus_dy:.1},{oz_plus_dz:.1}));
#42 = LINE('',#43,#44);
#43 = CARTESIAN_POINT('',({ox:.1},{oy_plus_dy:.1},{oz:.1}));
#44 = VECTOR('',#45,1.);
#45 = DIRECTION('',(0.,0.,1.));
#46 = ORIENTED_EDGE('',*,*,#47,.F.);
#47 = EDGE_CURVE('',#24,#40,#48,.T.);
#48 = LINE('',#49,#50);
#49 = CARTESIAN_POINT('',({ox:.1},{oy:.1},{oz_plus_dz:.1}));
#50 = VECTOR('',#51,1.);
#51 = DIRECTION('',(-0.,1.,0.));
#52 = PLANE('',#53);
#53 = AXIS2_PLACEMENT_3D('',#54,#55,#56);
#54 = CARTESIAN_POINT('',({ox:.1},{oy:.1},{oz:.1}));
#55 = DIRECTION('',(1.,0.,-0.));
#56 = DIRECTION('',(0.,0.,1.));
#57 = ADVANCED_FACE('',(#58),#92,.T.);
#58 = FACE_BOUND('',#59,.T.);
#59 = EDGE_LOOP('',(#60,#70,#78,#86));
#60 = ORIENTED_EDGE('',*,*,#61,.F.);
#61 = EDGE_CURVE('',#62,#64,#66,.T.);
#62 = VERTEX_POINT('',#63);
#63 = CARTESIAN_POINT('',({ox_plus_dx:.1},{oy:.1},{oz:.1}));
#64 = VERTEX_POINT('',#65);
#65 = CARTESIAN_POINT('',({ox_plus_dx:.1},{oy:.1},{oz_plus_dz:.1}));
#66 = LINE('',#67,#68);
#67 = CARTESIAN_POINT('',({ox_plus_dx:.1},{oy:.1},{oz:.1}));
#68 = VECTOR('',#69,1.);
#69 = DIRECTION('',(0.,0.,1.));
#70 = ORIENTED_EDGE('',*,*,#71,.T.);
#71 = EDGE_CURVE('',#62,#72,#74,.T.);
#72 = VERTEX_POINT('',#73);
#73 = CARTESIAN_POINT('',({ox_plus_dx:.1},{oy_plus_dy:.1},{oz:.1}));
#74 = LINE('',#75,#76);
#75 = CARTESIAN_POINT('',({ox_plus_dx:.1},{oy:.1},{oz:.1}));
#76 = VECTOR('',#77,1.);
#77 = DIRECTION('',(-0.,1.,0.));
#78 = ORIENTED_EDGE('',*,*,#79,.T.);
#79 = EDGE_CURVE('',#72,#80,#82,.T.);
#80 = VERTEX_POINT('',#81);
#81 = CARTESIAN_POINT('',({ox_plus_dx:.1},{oy_plus_dy:.1},{oz_plus_dz:.1}));
#82 = LINE('',#83,#84);
#83 = CARTESIAN_POINT('',({ox_plus_dx:.1},{oy_plus_dy:.1},{oz:.1}));
#84 = VECTOR('',#85,1.);
#85 = DIRECTION('',(0.,0.,1.));
#86 = ORIENTED_EDGE('',*,*,#87,.F.);
#87 = EDGE_CURVE('',#64,#80,#88,.T.);
#88 = LINE('',#89,#90);
#89 = CARTESIAN_POINT('',({ox_plus_dx:.1},{oy:.1},{oz_plus_dz:.1}));
#90 = VECTOR('',#91,1.);
#91 = DIRECTION('',(-0.,1.,0.));
#92 = PLANE('',#93);
#93 = AXIS2_PLACEMENT_3D('',#94,#95,#96);
#94 = CARTESIAN_POINT('',({ox_plus_dx:.1},{oy:.1},{oz:.1}));
#95 = DIRECTION('',(1.,0.,-0.));
#96 = DIRECTION('',(0.,0.,1.));
#97 = ADVANCED_FACE('',(#98),#114,.F.);
#98 = FACE_BOUND('',#99,.F.);
#99 = EDGE_LOOP('',(#100,#106,#107,#113));
#100 = ORIENTED_EDGE('',*,*,#101,.F.);
#101 = EDGE_CURVE('',#22,#62,#102,.T.);
#102 = LINE('',#103,#104);
#103 = CARTESIAN_POINT('',({ox:.1},{oy:.1},{oz:.1}));
#104 = VECTOR('',#105,1.);
#105 = DIRECTION('',(1.,0.,-0.));
#106 = ORIENTED_EDGE('',*,*,#21,.T.);
#107 = ORIENTED_EDGE('',*,*,#108,.T.);
#108 = EDGE_CURVE('',#24,#64,#109,.T.);
#109 = LINE('',#110,#111);
#110 = CARTESIAN_POINT('',({ox:.1},{oy:.1},{oz_plus_dz:.1}));
#111 = VECTOR('',#112,1.);
#112 = DIRECTION('',(1.,0.,-0.));
#113 = ORIENTED_EDGE('',*,*,#61,.F.);
#114 = PLANE('',#115);
#115 = AXIS2_PLACEMENT_3D('',#116,#117,#118);
#116 = CARTESIAN_POINT('',({ox:.1},{oy:.1},{oz:.1}));
#117 = DIRECTION('',(-0.,1.,0.));
#118 = DIRECTION('',(0.,0.,1.));
#119 = ADVANCED_FACE('',(#120),#136,.T.);
#120 = FACE_BOUND('',#121,.F.);
#121 = EDGE_LOOP('',(#122,#128,#129,#135));
#122 = ORIENTED_EDGE('',*,*,#123,.F.);
#123 = EDGE_CURVE('',#32,#72,#124,.T.);
#124 = LINE('',#125,#126);
#125 = CARTESIAN_POINT('',({ox:.1},{oy_plus_dy:.1},{oz:.1}));
#126 = VECTOR('',#127,1.);
#127 = DIRECTION('',(1.,0.,-0.));
#128 = ORIENTED_EDGE('',*,*,#39,.T.);
#129 = ORIENTED_EDGE('',*,*,#130,.T.);
#130 = EDGE_CURVE('',#40,#80,#131,.T.);
#131 = LINE('',#132,#133);
#132 = CARTESIAN_POINT('',({ox:.1},{oy_plus_dy:.1},{oz_plus_dz:.1}));
#133 = VECTOR('',#134,1.);
#134 = DIRECTION('',(1.,0.,-0.));
#135 = ORIENTED_EDGE('',*,*,#79,.F.);
#136 = PLANE('',#137);
#137 = AXIS2_PLACEMENT_3D('',#138,#139,#140);
#138 = CARTESIAN_POINT('',({ox:.1},{oy_plus_dy:.1},{oz:.1}));
#139 = DIRECTION('',(-0.,1.,0.));
#140 = DIRECTION('',(0.,0.,1.));
#141 = ADVANCED_FACE('',(#142),#148,.F.);
#142 = FACE_BOUND('',#143,.F.);
#143 = EDGE_LOOP('',(#144,#145,#146,#147));
#144 = ORIENTED_EDGE('',*,*,#31,.F.);
#145 = ORIENTED_EDGE('',*,*,#101,.T.);
#146 = ORIENTED_EDGE('',*,*,#71,.T.);
#147 = ORIENTED_EDGE('',*,*,#123,.F.);
#148 = PLANE('',#149);
#149 = AXIS2_PLACEMENT_3D('',#150,#151,#152);
#150 = CARTESIAN_POINT('',({ox:.1},{oy:.1},{oz:.1}));
#151 = DIRECTION('',(0.,0.,1.));
#152 = DIRECTION('',(1.,0.,-0.));
#153 = ADVANCED_FACE('',(#154),#160,.T.);
#154 = FACE_BOUND('',#155,.T.);
#155 = EDGE_LOOP('',(#156,#157,#158,#159));
#156 = ORIENTED_EDGE('',*,*,#47,.F.);
#157 = ORIENTED_EDGE('',*,*,#108,.T.);
#158 = ORIENTED_EDGE('',*,*,#87,.T.);
#159 = ORIENTED_EDGE('',*,*,#130,.F.);
#160 = PLANE('',#161);
#161 = AXIS2_PLACEMENT_3D('',#162,#163,#164);
#162 = CARTESIAN_POINT('',({ox_plus_dx:.1},{oy_plus_dy:.1},{oz_plus_dz:.1}));
#163 = DIRECTION('',(0.,0.,1.));
#164 = DIRECTION('',(1.,0.,-0.));
#165 = ( GEOMETRIC_REPRESENTATION_CONTEXT(3) GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((#169)) GLOBAL_UNIT_ASSIGNED_CONTEXT((#166,#167,#168)) REPRESENTATION_CONTEXT('Context #1','3D Context with UNIT and UNCERTAINTY') );
#166 = ( LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.) );
#167 = ( NAMED_UNIT(*) PLANE_ANGLE_UNIT() SI_UNIT($,.RADIAN.) );
#168 = ( NAMED_UNIT(*) SI_UNIT($,.STERADIAN.) SOLID_ANGLE_UNIT() );
#169 = UNCERTAINTY_MEASURE_WITH_UNIT(LENGTH_MEASURE(1.E-07),#166,'distance_accuracy_value','confusion accuracy');
#170 = PRODUCT_RELATED_PRODUCT_CATEGORY('part',$,(#7));
#171 = MECHANICAL_DESIGN_GEOMETRIC_PRESENTATION_REPRESENTATION('',(#172),#165);
#172 = STYLED_ITEM('color',(#173),#15);
#173 = PRESENTATION_STYLE_ASSIGNMENT((#174,#180));
#174 = SURFACE_STYLE_USAGE(.BOTH.,#175);
#175 = SURFACE_SIDE_STYLE('',(#176));
#176 = SURFACE_STYLE_FILL_AREA(#177);
#177 = FILL_AREA_STYLE('',(#178));
#178 = FILL_AREA_STYLE_COLOUR('',#179);
#179 = COLOUR_RGB('',0.678430976034,0.709803998361,0.741176010593);
#180 = CURVE_STYLE('',#181,POSITIVE_LENGTH_MEASURE(0.1),#182);
#181 = DRAUGHTING_PRE_DEFINED_CURVE_FONT('continuous');
#182 = COLOUR_RGB('',9.803921802644E-02,9.803921802644E-02,9.803921802644E-02);
ENDSEC;
END-ISO-10303-21;
"#,
       timestamp = timestamp,
       name = block.name,
       ox = ox,
       oy = oy,
       oz = oz,
       ox_plus_dx = ox_plus_dx,
       oy_plus_dy = oy_plus_dy,
       oz_plus_dz = oz_plus_dz
   );
    step_str
}
