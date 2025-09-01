# Getting Started with Kubernetes: A Hands-On Demonstration

This guide will walk you through a practical demonstration of the key features of Kubernetes. We will create a cluster, deploy an application, and then simulate node failures to observe how Kubernetes automatically maintains the desired state of our application.

## Core Kubernetes Features

### 1. Declarative Configuration & Desired State Management

Kubernetes works on a declarative model. You define the desired state of your application in a YAML manifest file, and Kubernetes works to achieve and maintain that state.

**Demonstration:**

*   **Create the Cluster:** We will first create a cluster of virtual machines that will form our Kubernetes cluster.
*   **Deploy an Application:** We will then deploy an Nginx web server using a `Deployment` object defined in `nginx-app.yaml`. This file tells Kubernetes we want to run a specific number of Nginx pods.

### 2. High Availability & Self-Healing

One of the most powerful features of Kubernetes is its ability to automatically recover from failures. If a worker node goes down, Kubernetes will automatically reschedule the pods that were running on that node to other healthy nodes in the cluster.

**Demonstration:**

*   **Simulate a Worker Node Failure:** We will stop one of the worker nodes in our cluster.
*   **Observe Self-Healing:** We will then observe how Kubernetes automatically creates new pods on the remaining worker nodes to maintain the desired number of replicas defined in our `Deployment`.
*   **Simulate a Worker Node Recovery:** We will start the worker node again and see how Kubernetes rebalances the cluster.

### 3. Scalability

Kubernetes makes it incredibly easy to scale your application up or down to meet demand. You can change the number of desired replicas in your `Deployment` and Kubernetes will automatically create or destroy pods to match.

**Demonstration:**

*   **Scale the Application:** We will increase the number of Nginx pods from 2 to 4 and observe how quickly Kubernetes provisions the new pods.

### 4. Service Discovery & Load Balancing

Kubernetes provides a stable way to access your application, even as pods are created and destroyed. A `Service` object provides a single, stable IP address and DNS name that will load balance traffic to the pods in a `Deployment`.

**Demonstration:**

*   **Expose the Application:** We will create a `Service` to expose our Nginx deployment within the cluster.
*   **Access the Application:** We will then show how you can access the Nginx web server using the `Service`'s IP address.

### 5. Ingress (Optional)

For exposing your application to the outside world, Kubernetes provides `Ingress` objects. An Ingress can provide HTTP and HTTPS routing to services within the cluster.

**Demonstration:**

*   **Create an Ingress:** We will create an `Ingress` object to route external traffic to our Nginx service.