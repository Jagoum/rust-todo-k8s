# These are just one time commands i am using to run my scripts

```sh
export PATH="/Users/gis/Library/Python/3.9/bin:$PATH" && ansible-playbook -i infra/ansible/inventory infra/ansible/deploy-linkerd.yml
```

```sh
export PATH="/Users/gis/Library/Python/3.9/bin:$PATH" && ansible-playbook -i infra/ansible/inventory infra/ansible/playbook.yml
```

```sh
export PATH="/Users/gis/Library/Python/3.9/bin:$PATH" && ansible -i infra/ansible/inventory controller-node -m command -a "kubectl get ns"
```

```sh
export PATH="/Users/gis/Library/Python/3.9/bin:$PATH" && ansible -i infra/ansible/inventory controller-node -m command -a "kubectl get deployments -n linkerd"
```

```sh
export PATH="/Users/gis/Library/Python/3.9/bin:$PATH" && ansible -i infra/ansible/inventory controller-node -m command -a "kubectl get deployments -n linkerd -o wide"

cd infra/ansible && export PATH="/Users/gis/Library/Python/3.9/bin:$PATH" && ansible-playbook -i inventory deploy-kubernetes.yml
```