class AddItemDictionary {
  const AddItemDictionary({
    required this.todo,
    required this.entry,
    required this.newTodo,
    required this.newEntry,
    required this.notes,
  });

  /// single word for todo
  final String todo;

  /// single word for shopping list entry
  final String entry;

  /// words for new todo
  final String newTodo;

  /// words for new shopping list entry
  final String newEntry;

  /// text hint for notes field
  final String notes;
}
