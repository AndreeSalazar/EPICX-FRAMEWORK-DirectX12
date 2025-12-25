// ADead-GPU Wave Intrinsics Test Shader
// NVIDIA RTX 3060 - Warp size 32
// Shader Model 6.0+
//
// This shader demonstrates wave/warp operations
// which are CRITICAL for GPU optimization

// Output buffer for results
RWStructuredBuffer<uint> OutputBuffer : register(u0);

// Wave intrinsics are available in SM 6.0+
// NVIDIA: Warp = 32 threads
// AMD:    Wave = 32 or 64 threads

[numthreads(256, 1, 1)]
void CSMain(uint3 dtid : SV_DispatchThreadID,
            uint3 gtid : SV_GroupThreadID,
            uint3 gid : SV_GroupID,
            uint gi : SV_GroupIndex) {
    
    // =========================================
    // WAVE QUERY INTRINSICS
    // =========================================
    
    // Get lane index within wave (0-31 for NVIDIA)
    uint laneIndex = WaveGetLaneIndex();
    
    // Get wave size (32 for NVIDIA)
    uint waveSize = WaveGetLaneCount();
    
    // Check if this is the first lane
    bool isFirstLane = WaveIsFirstLane();
    
    // =========================================
    // WAVE VOTE INTRINSICS
    // =========================================
    
    // True if ANY lane has condition true
    bool anyActive = WaveActiveAnyTrue(gtid.x < 16);
    
    // True if ALL lanes have condition true
    bool allActive = WaveActiveAllTrue(gtid.x < 256);
    
    // Ballot - returns bitmask of lanes where condition is true
    uint4 ballot = WaveActiveBallot(gtid.x % 2 == 0);
    
    // =========================================
    // WAVE REDUCTION INTRINSICS
    // =========================================
    
    // Sum across all active lanes
    uint waveSum = WaveActiveSum(gtid.x);
    
    // Product across all active lanes
    uint waveProd = WaveActiveProduct(1);  // Would be huge with real values
    
    // Min/Max across wave
    uint waveMin = WaveActiveMin(gtid.x);
    uint waveMax = WaveActiveMax(gtid.x);
    
    // Bitwise operations across wave
    uint waveAnd = WaveActiveBitAnd(0xFFFFFFFF);
    uint waveOr = WaveActiveBitOr(gtid.x);
    uint waveXor = WaveActiveBitXor(gtid.x);
    
    // Count active lanes
    uint activeCount = WaveActiveCountBits(true);
    
    // =========================================
    // WAVE SCAN (PREFIX) INTRINSICS
    // =========================================
    
    // Prefix sum (exclusive - doesn't include current lane)
    uint prefixSum = WavePrefixSum(1);
    
    // Prefix product
    uint prefixProd = WavePrefixProduct(1);
    
    // =========================================
    // WAVE SHUFFLE INTRINSICS
    // =========================================
    
    // Read from specific lane
    uint fromLane0 = WaveReadLaneAt(gtid.x, 0);
    
    // Read from first active lane
    uint fromFirst = WaveReadLaneFirst(gtid.x);
    
    // =========================================
    // OUTPUT RESULTS (first thread of each wave)
    // =========================================
    
    if (isFirstLane) {
        uint waveIndex = gi / waveSize;
        uint baseOffset = waveIndex * 16;
        
        OutputBuffer[baseOffset + 0] = waveSize;
        OutputBuffer[baseOffset + 1] = waveSum;
        OutputBuffer[baseOffset + 2] = waveMin;
        OutputBuffer[baseOffset + 3] = waveMax;
        OutputBuffer[baseOffset + 4] = activeCount;
        OutputBuffer[baseOffset + 5] = ballot.x;
        OutputBuffer[baseOffset + 6] = fromLane0;
        OutputBuffer[baseOffset + 7] = prefixSum;
    }
    
    // =========================================
    // TYPICAL WAVE-EFFICIENT PATTERN
    // Parallel reduction example
    // =========================================
    
    // Example: Find sum of all thread indices
    // This is MUCH faster than using shared memory
    uint myValue = dtid.x;
    uint totalSum = WaveActiveSum(myValue);
    
    // Only one thread per wave writes result
    if (isFirstLane) {
        // Atomic add for combining wave results
        uint dummy;
        InterlockedAdd(OutputBuffer[1000], totalSum, dummy);
    }
}

// =========================================
// SIMPLE VERSION for testing
// =========================================

[numthreads(32, 1, 1)]
void CSSimple(uint3 dtid : SV_DispatchThreadID) {
    // Each thread writes its lane index
    // Should produce 0,1,2,3...31 pattern for NVIDIA
    uint laneIndex = WaveGetLaneIndex();
    OutputBuffer[dtid.x] = laneIndex;
}

