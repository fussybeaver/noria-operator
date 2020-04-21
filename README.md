# Noria-operator: a kubernetes operator for the Noria database

This operator brings high availability with stateful persistence on a clustered environment to the [high performance database engine Noria](https://github.com/mit-pdos/noria).

## Getting started

You can install the operator using helm:

```
helm repo add fussybeaver 'https://raw.githubusercontent.com/fussybeaver/noria-operator/master/helm'
helm repo update
helm install generic fussybeaver/noria-operator
```

Then adjust and apply a manifest such as the sample manifest in this repository.

```
kubectl apply -f 'https://raw.githubusercontent.com/fussybeaver/noria-operator/master/manifest/simple.yaml'
```

