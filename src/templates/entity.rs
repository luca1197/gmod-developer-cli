/*
	Basic physics entity
*/
pub static ENTITY_BASIC_CL: &str = r#"include("shared.lua")

function ENT:Initialize()

end
"#;

pub static ENTITY_BASIC_SV: &str = r#"AddCSLuaFile("cl_init.lua")
AddCSLuaFile("shared.lua")
include("shared.lua")

function ENT:Initialize()
	self:SetModel("%MODEL%")
	self:PhysicsInit(SOLID_VPHYSICS)
	self:SetMoveType(MOVETYPE_VPHYSICS)
	self:SetSolid(SOLID_VPHYSICS)
	self:SetUseType(SIMPLE_USE)
end
"#;

pub static ENTITY_BASIC_SH: &str = r#"ENT.Type = "anim"
ENT.Base = "base_anim"

ENT.PrintName = "%PRINTNAME%"
ENT.Category = "%CATEGORY%"
ENT.Author = "%AUTHOR%"
ENT.Spawnable = %SPAWNABLE%
"#;

/*
	NPC
*/
pub static ENTITY_NPC_CL: &str = r#"include("shared.lua")

function ENT:Initialize()

end
"#;

pub static ENTITY_NPC_SV: &str = r#"AddCSLuaFile("cl_init.lua")
AddCSLuaFile("shared.lua")
include("shared.lua")

function ENT:Initialize()
	self:SetModel("%MODEL%")
	self:SetSolid(SOLID_BBOX)
	self:SetHullSizeNormal()
	self:SetNPCState(NPC_STATE_IDLE)
	self:SetHullType(HULL_HUMAN)
	self:SetUseType(SIMPLE_USE)
	self:CapabilitiesAdd(CAP_ANIMATEDFACE)
	self:CapabilitiesAdd(CAP_TURN_HEAD)
	self:DropToFloor()
end

function ENT:Use(activator)
	
	if not activator:IsPlayer() then return end

end"#;

pub static ENTITY_NPC_SH: &str = r#"ENT.Type = "ai"
ENT.Base = "base_ai"

ENT.PrintName = "%PRINTNAME%"
ENT.Category = "%CATEGORY%"
ENT.Author = "%AUTHOR%"
ENT.Spawnable = %SPAWNABLE%

ENT.RenderGroup = RENDERGROUP_TRANSLUCENT
ENT.AutomaticFrameAdvance = true
"#;