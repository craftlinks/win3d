float4 VSMain( float2 pos : Position ) : SV_Position
{
	return float4(pos.x,pos.y,0.0f,1.0f);
}

float4 PSMain() : SV_Target
{
    return float4(1.0f, 1.0f, 1.0f, 1.0f);
}