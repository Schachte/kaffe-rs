import { useState } from "react";

const fn = (click: number) => {
  return <div>Button clicked: {click} times</div>;
};

const Home = () => {
  const [click, setClick] = useState(0);

  return (
    <div>
      {fn(click)}
      <button onClick={() => setClick(click + 1)}>YAY</button>
    </div>
  );
};

export default Home;
