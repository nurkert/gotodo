class TitlesDictionary {
  const TitlesDictionary({
    required this.todos,
    required this.shoppingList,
    required this.lookUp,
    required this.notes,
    required this.settings,
    required this.settingsGeneral,
  });

  /// description of the todos tab in the bottom navigation bar
  final String todos;

  /// description of the shopping list tab in the bottom navigation bar
  final String shoppingList;

  /// description of the online tab in the bottom navigation bar
  final String lookUp;

  final String notes;

  /// description of the settings tab in the bottom navigation bar
  final String settings;

  /// the title in the app bar for general settings
  final String settingsGeneral;
}
