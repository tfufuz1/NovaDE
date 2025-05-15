#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include "core/services/ServiceRegistry.h"

using namespace VivoX::Core::Services;
using namespace testing;

// Mock service implementation
class MockService : public Service {
public:
    MockService(const std::string& id, const std::string& name)
        : m_id(id), m_name(name) {}
    
    std::string getId() const override { return m_id; }
    std::string getName() const override { return m_name; }
    std::vector<std::string> getDependencies() const override { return m_dependencies; }
    
    bool initialize() override {
        m_initialized = true;
        return true;
    }
    
    bool shutdown() override {
        m_initialized = false;
        return true;
    }
    
    bool isInitialized() const override { return m_initialized; }
    
    void addDependency(const std::string& dependency) {
        m_dependencies.push_back(dependency);
    }
    
private:
    std::string m_id;
    std::string m_name;
    std::vector<std::string> m_dependencies;
    bool m_initialized = false;
};

class ServiceRegistryTest : public Test {
protected:
    void SetUp() override {
        // Initialize service registry
        ServiceRegistry::initialize();
    }

    void TearDown() override {
        ServiceRegistry::shutdown();
    }
    
    // Helper to create and register a mock service
    std::shared_ptr<MockService> createAndRegisterService(
        const std::string& id, 
        const std::string& name,
        const std::vector<std::string>& dependencies = {}) {
        
        auto service = std::make_shared<MockService>(id, name);
        
        // Add dependencies
        for (const auto& dep : dependencies) {
            service->addDependency(dep);
        }
        
        // Register service
        ServiceRegistry::registerService(service);
        
        return service;
    }
};

TEST_F(ServiceRegistryTest, InitializationTest) {
    EXPECT_TRUE(ServiceRegistry::isInitialized());
}

TEST_F(ServiceRegistryTest, RegisterServiceTest) {
    auto service = std::make_shared<MockService>("test.service", "Test Service");
    
    // Register service
    EXPECT_TRUE(ServiceRegistry::registerService(service));
    
    // Try to register the same service again (should fail)
    EXPECT_FALSE(ServiceRegistry::registerService(service));
}

TEST_F(ServiceRegistryTest, GetServiceTest) {
    auto service = createAndRegisterService("test.service", "Test Service");
    
    // Get service by ID
    auto retrievedService = ServiceRegistry::getService("test.service");
    EXPECT_NE(retrievedService, nullptr);
    EXPECT_EQ(retrievedService->getId(), "test.service");
    EXPECT_EQ(retrievedService->getName(), "Test Service");
    
    // Try to get non-existent service
    EXPECT_EQ(ServiceRegistry::getService("non.existent.service"), nullptr);
}

TEST_F(ServiceRegistryTest, GetAllServicesTest) {
    // Register multiple services
    auto service1 = createAndRegisterService("service1", "Service 1");
    auto service2 = createAndRegisterService("service2", "Service 2");
    auto service3 = createAndRegisterService("service3", "Service 3");
    
    // Get all services
    auto services = ServiceRegistry::getAllServices();
    
    // Verify all services are in the list
    EXPECT_EQ(services.size(), 3);
    
    // Check if all services are in the list
    bool hasService1 = false, hasService2 = false, hasService3 = false;
    
    for (const auto& service : services) {
        if (service->getId() == "service1") hasService1 = true;
        if (service->getId() == "service2") hasService2 = true;
        if (service->getId() == "service3") hasService3 = true;
    }
    
    EXPECT_TRUE(hasService1);
    EXPECT_TRUE(hasService2);
    EXPECT_TRUE(hasService3);
}

TEST_F(ServiceRegistryTest, UnregisterServiceTest) {
    auto service = createAndRegisterService("test.service", "Test Service");
    
    // Unregister service
    EXPECT_TRUE(ServiceRegistry::unregisterService("test.service"));
    
    // Verify service is no longer registered
    EXPECT_EQ(ServiceRegistry::getService("test.service"), nullptr);
    
    // Try to unregister non-existent service
    EXPECT_FALSE(ServiceRegistry::unregisterService("non.existent.service"));
}

TEST_F(ServiceRegistryTest, InitializeServiceTest) {
    auto service = createAndRegisterService("test.service", "Test Service");
    
    // Initialize service
    EXPECT_TRUE(ServiceRegistry::initializeService("test.service"));
    
    // Verify service is initialized
    EXPECT_TRUE(service->isInitialized());
    
    // Try to initialize non-existent service
    EXPECT_FALSE(ServiceRegistry::initializeService("non.existent.service"));
}

TEST_F(ServiceRegistryTest, ShutdownServiceTest) {
    auto service = createAndRegisterService("test.service", "Test Service");
    
    // Initialize service
    ServiceRegistry::initializeService("test.service");
    
    // Shutdown service
    EXPECT_TRUE(ServiceRegistry::shutdownService("test.service"));
    
    // Verify service is not initialized
    EXPECT_FALSE(service->isInitialized());
    
    // Try to shutdown non-existent service
    EXPECT_FALSE(ServiceRegistry::shutdownService("non.existent.service"));
}

TEST_F(ServiceRegistryTest, InitializeAllServicesTest) {
    // Register multiple services
    auto service1 = createAndRegisterService("service1", "Service 1");
    auto service2 = createAndRegisterService("service2", "Service 2");
    auto service3 = createAndRegisterService("service3", "Service 3");
    
    // Initialize all services
    EXPECT_TRUE(ServiceRegistry::initializeAllServices());
    
    // Verify all services are initialized
    EXPECT_TRUE(service1->isInitialized());
    EXPECT_TRUE(service2->isInitialized());
    EXPECT_TRUE(service3->isInitialized());
}

TEST_F(ServiceRegistryTest, ShutdownAllServicesTest) {
    // Register multiple services
    auto service1 = createAndRegisterService("service1", "Service 1");
    auto service2 = createAndRegisterService("service2", "Service 2");
    auto service3 = createAndRegisterService("service3", "Service 3");
    
    // Initialize all services
    ServiceRegistry::initializeAllServices();
    
    // Shutdown all services
    EXPECT_TRUE(ServiceRegistry::shutdownAllServices());
    
    // Verify all services are not initialized
    EXPECT_FALSE(service1->isInitialized());
    EXPECT_FALSE(service2->isInitialized());
    EXPECT_FALSE(service3->isInitialized());
}

TEST_F(ServiceRegistryTest, DependencyResolutionTest) {
    // Create services with dependencies
    auto serviceA = createAndRegisterService("service.a", "Service A");
    auto serviceB = createAndRegisterService("service.b", "Service B", {"service.a"});
    auto serviceC = createAndRegisterService("service.c", "Service C", {"service.b"});
    auto serviceD = createAndRegisterService("service.d", "Service D", {"service.a", "service.c"});
    
    // Initialize all services
    EXPECT_TRUE(ServiceRegistry::initializeAllServices());
    
    // Verify all services are initialized
    EXPECT_TRUE(serviceA->isInitialized());
    EXPECT_TRUE(serviceB->isInitialized());
    EXPECT_TRUE(serviceC->isInitialized());
    EXPECT_TRUE(serviceD->isInitialized());
}

TEST_F(ServiceRegistryTest, CircularDependencyDetectionTest) {
    // Create services with circular dependencies
    auto serviceA = createAndRegisterService("service.a", "Service A", {"service.c"});
    auto serviceB = createAndRegisterService("service.b", "Service B", {"service.a"});
    auto serviceC = createAndRegisterService("service.c", "Service C", {"service.b"});
    
    // Try to initialize all services (should fail due to circular dependencies)
    EXPECT_FALSE(ServiceRegistry::initializeAllServices());
    
    // Verify no services are initialized
    EXPECT_FALSE(serviceA->isInitialized());
    EXPECT_FALSE(serviceB->isInitialized());
    EXPECT_FALSE(serviceC->isInitialized());
}

TEST_F(ServiceRegistryTest, MissingDependencyTest) {
    // Create service with missing dependency
    auto service = createAndRegisterService("test.service", "Test Service", {"missing.dependency"});
    
    // Try to initialize service (should fail due to missing dependency)
    EXPECT_FALSE(ServiceRegistry::initializeService("test.service"));
    
    // Verify service is not initialized
    EXPECT_FALSE(service->isInitialized());
}

TEST_F(ServiceRegistryTest, ServiceCallbackTest) {
    bool onRegisterCalled = false;
    bool onUnregisterCalled = false;
    bool onInitializeCalled = false;
    bool onShutdownCalled = false;
    
    // Register callbacks
    ServiceRegistry::registerServiceCallback(ServiceCallback::OnRegister, [&onRegisterCalled](const std::string& serviceId) {
        onRegisterCalled = true;
        return true;
    });
    
    ServiceRegistry::registerServiceCallback(ServiceCallback::OnUnregister, [&onUnregisterCalled](const std::string& serviceId) {
        onUnregisterCalled = true;
        return true;
    });
    
    ServiceRegistry::registerServiceCallback(ServiceCallback::OnInitialize, [&onInitializeCalled](const std::string& serviceId) {
        onInitializeCalled = true;
        return true;
    });
    
    ServiceRegistry::registerServiceCallback(ServiceCallback::OnShutdown, [&onShutdownCalled](const std::string& serviceId) {
        onShutdownCalled = true;
        return true;
    });
    
    // Create and register service
    auto service = std::make_shared<MockService>("test.service", "Test Service");
    ServiceRegistry::registerService(service);
    
    // Verify OnRegister callback was called
    EXPECT_TRUE(onRegisterCalled);
    
    // Initialize service
    ServiceRegistry::initializeService("test.service");
    
    // Verify OnInitialize callback was called
    EXPECT_TRUE(onInitializeCalled);
    
    // Shutdown service
    ServiceRegistry::shutdownService("test.service");
    
    // Verify OnShutdown callback was called
    EXPECT_TRUE(onShutdownCalled);
    
    // Unregister service
    ServiceRegistry::unregisterService("test.service");
    
    // Verify OnUnregister callback was called
    EXPECT_TRUE(onUnregisterCalled);
}

TEST_F(ServiceRegistryTest, TopologicalSortTest) {
    // Create services with dependencies in reverse order
    auto serviceD = createAndRegisterService("service.d", "Service D", {"service.b", "service.c"});
    auto serviceC = createAndRegisterService("service.c", "Service C", {"service.a"});
    auto serviceB = createAndRegisterService("service.b", "Service B", {"service.a"});
    auto serviceA = createAndRegisterService("service.a", "Service A");
    
    // Get sorted services
    auto sortedServices = ServiceRegistry::getSortedServices();
    
    // Verify order is correct (A should be first, D should be last)
    ASSERT_EQ(sortedServices.size(), 4);
    EXPECT_EQ(sortedServices[0]->getId(), "service.a");
    EXPECT_TRUE(sortedServices[3]->getId() == "service.d");
    
    // B and C can be in either order, but both must come after A and before D
    EXPECT_TRUE(sortedServices[1]->getId() == "service.b" || sortedServices[1]->getId() == "service.c");
    EXPECT_TRUE(sortedServices[2]->getId() == "service.b" || sortedServices[2]->getId() == "service.c");
}

int main(int argc, char **argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}
