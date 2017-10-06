
use std::sync::Arc;
use std::ptr;
use std::marker::PhantomData;
use smallvec::SmallVec;
use vks;
use ::{util, VooResult, Device, Handle, CommandPoolCreateInfo, CommandPoolCreateFlags,
    CommandBufferAllocateInfo, CommandBufferHandle, CommandBufferLevel, CommandBuffer};


#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct CommandPoolHandle(pub vks::VkCommandPool);

impl Handle for CommandPoolHandle {
    type Target = CommandPoolHandle;

    fn handle(&self) -> Self::Target {
        *self
    }
}


#[derive(Debug)]
struct Inner {
    handle: CommandPoolHandle,
    device: Device,
}

#[derive(Debug, Clone)]
pub struct CommandPool {
    inner: Arc<Inner>,
}

impl CommandPool {
    /// Returns a new `CommandPoolBuilder`.
    pub fn builder<'b>() -> CommandPoolBuilder<'b> {
        CommandPoolBuilder::new()
    }

    pub fn allocate_command_buffer_handles(&self, level: CommandBufferLevel, count: u32)
            -> VooResult<SmallVec<[CommandBufferHandle; 16]>> {
        let alloc_info = CommandBufferAllocateInfo::builder()
            .command_pool(self.handle())
            .level(level)
            .command_buffer_count(count)
            .build();

        let mut command_buffers: SmallVec<[CommandBufferHandle; 16]> = SmallVec::new();
        command_buffers.reserve_exact(count as usize);
        unsafe {
            command_buffers.set_len(count as usize);
            ::check(self.inner.device.proc_addr_loader().core.vkAllocateCommandBuffers(
                self.inner.device.handle().0, alloc_info.as_raw(),
                command_buffers.as_mut_ptr() as *mut vks::VkCommandBuffer));
        }
        Ok(command_buffers)
    }

    pub fn allocate_command_buffers(&self, level: CommandBufferLevel, count: u32)
            -> VooResult<SmallVec<[CommandBuffer; 16]>> {
        self.allocate_command_buffer_handles(level, count)?.iter().map(|&hndl| {
            CommandBuffer::from_parts(self.clone(), hndl)
        }).collect::<Result<SmallVec<_>, _>>()
    }

    pub fn allocate_command_buffer(&self, level: CommandBufferLevel) -> VooResult<CommandBuffer> {
        self.allocate_command_buffers(level, 1).map(|mut cbs| cbs.remove(0))
    }

    pub fn handle(&self) -> CommandPoolHandle {
        self.inner.handle
    }

    /// Returns a reference to the associated device.
    pub fn device(&self) -> &Device {
        &self.inner.device
    }
}

impl<'h> Handle for &'h CommandPool {
    type Target = CommandPoolHandle;

    fn handle(&self) -> Self::Target {
        self.inner.handle
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        unsafe {
            self.device.proc_addr_loader().core.vkDestroyCommandPool(self.device.handle().0,
                self.handle.0, ptr::null());
        }
    }
}


/// A builder for `CommandPool`.
#[derive(Debug, Clone)]
pub struct CommandPoolBuilder<'b> {
    create_info: CommandPoolCreateInfo<'b>,
    _p: PhantomData<&'b ()>,
}

impl<'b> CommandPoolBuilder<'b> {
    /// Returns a new render pass builder.
    pub fn new() -> CommandPoolBuilder<'b> {
        CommandPoolBuilder {
            create_info: CommandPoolCreateInfo::default(),
            _p: PhantomData,
        }
    }

    /// Specifies the usage behavior for the pool and command buffers
    /// allocated from it.
    pub fn flags<'s>(&'s mut self, flags: CommandPoolCreateFlags)
            -> &'s mut CommandPoolBuilder<'b> {
        self.create_info.set_flags(flags);
        self
    }

    /// Specifies a queue family.
    ///
    /// All command buffers allocated from this command pool must be submitted
    /// on queues from the same queue family.
    pub fn queue_family_index<'s>(&'s mut self, queue_family_index: u32)
            -> &'s mut CommandPoolBuilder<'b> {
        self.create_info.set_queue_family_index(queue_family_index);
        self
    }

    /// Creates and returns a new `CommandPool`
    pub fn build(&self, device: Device) -> VooResult<CommandPool> {
        let mut handle = 0;
        unsafe {
            ::check(device.proc_addr_loader().core.vkCreateCommandPool(device.handle().0,
                self.create_info.as_raw(), ptr::null(), &mut handle));
        }

        Ok(CommandPool {
            inner: Arc::new(Inner {
                handle: CommandPoolHandle(handle),
                device,
            })
        })
    }
}





