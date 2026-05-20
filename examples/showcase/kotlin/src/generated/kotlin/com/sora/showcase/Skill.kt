package com.sora.showcase

data class Skill(
    val id: Int,
    val name: String,
    val element: ElementType,
    val cost: ResourceCost,
    val effect: SkillEffect,
    val requiredLevel: Int,
    val requiredItem: Int?,
    val castOrigin: Vec3,
) {
    companion object {
        fun decode(reader: SoraReader): Skill =
            Skill(
                id = reader.readI32(),
                name = reader.readString(),
                element = ElementType.decode(reader),
                cost = ResourceCost.decode(reader),
                effect = SkillEffect.decode(reader),
                requiredLevel = reader.readI32(),
                requiredItem = reader.readOptional { reader.readI32() },
                castOrigin = Vec3.decode(reader),
            )
    }
}
