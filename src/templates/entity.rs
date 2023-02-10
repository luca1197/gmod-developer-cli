pub static ENTITY_BASIC_CL: &str = r#"include("shared.lua")

function ENT:Initialize()

end"#;

pub static ENTITY_BASIC_SV: &str = r#"AddCSLuaFile("cl_init.lua")
AddCSLuaFile("shared.lua")
include("shared.lua")

function ENT:Initialize()
	self:SetModel("models/policerp/bankrobbery/pile.mdl")
	self:PhysicsInit(SOLID_VPHYSICS)
	self:SetMoveType(MOVETYPE_VPHYSICS)
	self:SetSolid(SOLID_VPHYSICS)
	self:SetUseType(SIMPLE_USE)
end
"#;

pub static ENTITY_BASIC_SH: &str = r#"ENT.Type = "anim"
ENT.Base = "base_anim"
ENT.Category = "%CATEGORY%"

ENT.Spawnable = %SPAWNABLE%
ENT.PrintName = "%PRINTNAME%"
ENT.Author = "%AUTHOR%""#;