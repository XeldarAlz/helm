using System;

namespace Game.Logic.Events
{
    public interface IEventBus
    {
        IDisposable Subscribe<T>(Action<T> handler) where T : struct, IEvent;
        void Unsubscribe<T>(Action<T> handler) where T : struct, IEvent;
        void Publish<T>(T eventData) where T : struct, IEvent;
        void Clear();
    }

    public interface IEvent { }
}
