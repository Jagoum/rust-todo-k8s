

   1. Provision and Harden VMs:

    ```sh
    ANSIBLE_CONFIG=infra/ansible/ansible.cfg ansible-playbook -i infra/ansible/hosts.ini
     infra/ansible/playbook.yml
    ```

   2. Set Up Local Docker Registry:

    ```sh
     ANSIBLE_CONFIG=infra/ansible/ansible.cfg ansible-playbook -i infra/ansible/hosts.ini
     infra/ansible/registry-playbook.yml
    ````

   3. Install k3s on Master Node:
 
    ```sh
     ANSIBLE_CONFIG=infra/ansible/ansible.cfg ansible-playbook -i infra/ansible/hosts.ini
     infra/ansible/k3s-playbook.yml
    ```

   4. Join Worker Node:

     ```sh
     ANSIBLE_CONFIG=infra/ansible/ansible.cfg ansible-playbook -i infra/ansible/hosts.ini
     infra/ansible/join-workers.yml
     ```

  After running these playbooks, your local Docker registry will be available on machine-1's IP address at port
  5000. You can then push and pull images to this registry from any of your VMs.
