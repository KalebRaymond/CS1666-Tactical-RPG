pub enum PlayerAction {
	Default,
	ChoosingUnitAction,
	MovingUnit,
	AttackingUnit,
	ChoosingNewUnit,

	// Specifically for choosing the new class to add to team
	ChoosePrimer,
	ChosenRanger,
	ChosenMelee,
	ChosenMage,
}