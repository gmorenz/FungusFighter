How to extract timings for normal attacks:

Footsies/Assets/Fighter/F00/Actions/N_Attack.asset
    -> MonoBehavior -> motions ->
       startEndFrame is a range of frame numbers we think
       motionID references ID's in:

Footsies/Assets/Fighter/F00/F00_MotionDataCotainer.asset
    -> MonoBehavior -> motionDataList -> motionID: <N> -> sprite -> fileID

Footsies/Assets/Fighter/F00/F00_Attack_0.png.meta
    -> TextureImporter -> fileIDToRecycleName -> <N> -> F00_Attack_0_<sprite_idx>

Hurtbox/Hitboxes in
Footsies/Assets/Fighter/F00/Actions/N_Attack.asset

Hitbox specifies attackID

AttackID in footsies/Assets/Fighter/F00/F00_AttackDataContainer.asset
hitStunFrame/guardStunFrame/guardBreakStunFrame frame data
and damageActionID/guardActionID

ActionsID's are pointers into Actions/N_Attack.asset and the like
