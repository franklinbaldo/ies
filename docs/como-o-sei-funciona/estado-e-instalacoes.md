---
type: SEIObservation
title: Estado e instalações do SEI
description: Diferenças de versões e personalizações observadas
---

## Estado e instalações

Em sete cassettes de caixa de processos verificadas por SHA-1/SHA-256,
a unidade da barra do SEI aparece no texto de `lnkInfraUnidade`:

| Instalação | Rótulo da barra |
| --- | --- |
| [UnB](../../cassettes/unb/unidade-processos-paginado-unb--e02dbcfde8c4.md) | Unidade4 |
| [UFRJ](../../cassettes/ufrj/unidade-processos-paginado-ufrj--1931211421b4.md) | ADCENTRAL |
| [Goiás](../../cassettes/go/unidade-processos-paginado-go--ca25324d2aac.md) | SEAD/CDE-09516 |
| [PEN1](../../cassettes/pen1/unidade-processos-paginado-pen1--11033ecce8d5.md) | TESTE |
| [PEN2](../../cassettes/pen2/unidade-processos-paginado-pen2--2e5309581805.md) | TESTE |
| [PEN3](../../cassettes/pen3/unidade-processos-paginado-pen3--85d8066a3934.md) | TESTE |
| [PEN4](../../cassettes/pen4/unidade-processos-paginado-pen4--cbedd235c299.md) | TESTE |

O elemento é um `<a>` sem `href` próprio nesses exemplos.
Seu texto é **rótulo visual da unidade ativa**, não ID da unidade
nem prova da identidade da sessão. Pode mudar quando o usuário
troca a unidade. O IES usa esse texto para conferir `--unidade`
e mesclas de páginas, nunca para construir links assinados.

Unidade ativa, modo de visualização, filtros e página corrente
podem persistir na sessão. No histórico de andamentos,
`hdnTipoHistorico` distingue modo resumido e completo.
A caixa sinaliza processos visualizados por classes CSS.

A versão SEI é metadado do ambiente e **não certificado de
compatibilidade**. Duas instalações com o mesmo número de versão
podem divergir por módulos instalados, opções, órgão e permissões.
A [separação entre instalação, órgão, unidade e versão](../interfaces/instalacoes-e-versoes.md)
é documentada sem criar um subsistema de pesquisa no runtime.

As instalações públicas mapeadas em outubro de 2026 foram
UnB (SEI 4.0.9), UFRJ (4.1.5), Goiás (5.0.3),
PEN1 (4.0.12), PEN2 (4.1.5), PEN3 e PEN4 (5.0.4).
Versões são medições daquela data, não informações permanentes.

Variam login, nomenclatura de protocolo, caixa resumida,
`data-label`, listas e ações oferecidas. Diferentes
instalações da mesma versão podem divergir por customização.

Uma recusa porque **a ação não apareceu para aquele recurso**
não comprova ausência da capacidade em toda a instalação.
Antes de atribuir comportamento à versão, compare com outra
instalação da mesma versão e com uma de versão diferente.
