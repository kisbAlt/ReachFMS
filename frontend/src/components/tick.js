export function Tick(props) {
    return (
        <svg style={{height: "20px", marginLeft: "10px", marginRight: "10px"}} viewBox="2.426 10.22 253.44 185.074" xmlns="http://www.w3.org/2000/svg">
        <rect x="46.424" y="64.385" width="25" height="121.521" fill={props.color} transform="matrix(0.707115, -0.707099, 0.707099, 0.707115, -75.927751, 96.663408)" rx="7.213" ry="7.213"/>
        <rect x="46.425" y="124.997" width="25" height="235.919" rx="7.213" ry="7.213" fill={props.color} transform="matrix(0.707099, 0.707115, -0.707115, 0.707099, 293.748265, -110.993204)"/>
        </svg>
    )
}