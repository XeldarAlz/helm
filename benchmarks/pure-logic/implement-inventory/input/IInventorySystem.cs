namespace Game.Logic.Inventory
{
    public interface IInventorySystem
    {
        bool AddItem(int itemId, int amount);
        bool RemoveItem(int itemId, int amount);
        int GetItemCount(int itemId);
        bool HasItem(int itemId, int amount = 1);
        void Clear();
    }

    public interface IInventoryConfig
    {
        int MaxSlots { get; }
        int MaxStackSize { get; }
    }

    public interface IInventoryEvents
    {
        void OnItemAdded(int itemId, int amount, int newTotal);
        void OnItemRemoved(int itemId, int amount, int newTotal);
        void OnInventoryCleared();
    }

    public readonly record struct InventorySlot(int ItemId, int Count);
}
