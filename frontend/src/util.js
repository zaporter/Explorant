
import React, {useEffect, useState} from 'react';

export const useRemoteResource = (defaultVal, requestBody, endpoint, effectHook=[]) => {
  const [count, setCount] = useState(defaultVal);
  const requestOptions = {
    method: 'POST',
    headers: {'Content-Type': 'application/json'},
    body: JSON.stringify(requestBody)
  };
  useEffect(()=>{
    fetch('http://127.0.0.1:8080/'+endpoint,requestOptions)
      .then(response=>response.json())
      .then(data=>setCount(data))
  },effectHook);
  return [count,setCount];
};
export const callRemote = (requestBody, endpoint) => {
  const requestOptions = {
    method: 'POST',
    headers: {'Content-Type': 'application/json'},
    body: JSON.stringify(requestBody)
  };
  return fetch('http://127.0.0.1:8080/'+endpoint,requestOptions)
};
