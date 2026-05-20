package com.sora.showcase

data class SkillEffect(
    val element: ElementType,
    val power: Int,
    val radius: Float,
) {
    companion object {
        fun decode(reader: SoraReader): SkillEffect =
            SkillEffect(
                element = ElementType.decode(reader),
                power = reader.readI32(),
                radius = reader.readF32(),
            )
    }
}