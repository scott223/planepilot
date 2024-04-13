"use client"; // This is a client component 👈🏽
import React, { useRef, useState, useEffect } from 'react'

function ChannelViewer() {
  const [count, setCount] = useState(0);

  // Similar to componentDidMount and componentDidUpdate:
  useEffect(() => {
    // First, we need to create an instance of EventSource and pass the data stream URL as a
    // parameter in its constructor
    const es = new EventSource("http://localhost:3000/api/v1/channel/1/stream");
    // Whenever the connection is established between the server and the client we'll get notified
    es.onopen = () => console.log(">>> Connection opened!");
    // Made a mistake, or something bad happened on the server? We get notified here
    es.onerror = (e) => console.log("ERROR!", e);
    // This is where we get the messages. The event is an object and we're interested in its `data` property
    es.onmessage = (e) => {
      console.log(">>>", e.data);
    };
    // Whenever we're done with the data stream we must close the connection
    return () => es.close();
  });

  return (
    <div>
      <p>You clicked {count} times</p>
      <button onClick={() => setCount(count + 1)}>
        Click me
      </button>
    </div>
  );
}

export default function Home() {

  return (
    <main className="flex min-h-screen flex-col items-center justify-between">
      <div className="border-solid border-2 border-indigo-600 w-full h-screen">
        <ChannelViewer />
      </div>
    </main>
  );
}
