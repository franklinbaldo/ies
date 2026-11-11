---
name: reverse-engineer-anything
type: Guide
title: "Reverse Engineer Anything — skill REA do repositório"
description: "Investigar comportamento de artefatos, HAR e runtime autorizado com REA; distinguir fatos, hipóteses e desconhecidos"
metadata:
  version: "61"
---

# REA — Reverse Engineer Anything

Skill **incluída no repositório** com instruções integrais do upstream
repartidas em arquivos menores para cumprir o limite de 20 KB/arquivo.
Fonte: [morluto/rea](https://github.com/morluto/rea/tree/25d3c7237ffeffc4f9e6ab87be2f83a643cf5cbb/.agents/skills/reverse-engineer-anything)
na revisão `25d3c7237ffeffc4f9e6ab87be2f83a643cf5cbb` (versão 61).

## Instruções completas (leitura obrigatória em ordem)

1. [Parte 1](references/upstream-part-1.md)
2. [Parte 2](references/upstream-part-2.md)
3. [Parte 3](references/upstream-part-3.md)
4. [Parte 4](references/upstream-part-4.md)

Para investigação Web do anti-IES, use primeiro a captura HAR/SAZ
fornecida ou runtime de treinamento explicitamente autorizado;
`inspect_web_network_capture` para captura passiva, nunca cassette
JSON como se fosse HAR. O guia local
[REA no anti-IES](../../../docs/anti-ies/rea-engenharia-reversa.md)
estabelece os limites. A skill contém instruções, **não instala**
automaticamente o servidor MCP, seus providers ou executáveis externos.
A ausência dessas ferramentas não bloqueia RED/GREEN sintético.

Não reproduza a implementação interna do SEI quando bastar simular
um efeito e emular apenas suas propriedades observáveis. Separe fato,
hipótese e desconhecido; não alegue REA executado sem ter usado REA.

## Aplicação ao E2E do IES

Desde 11/11/2026, use [cassette-e2e](../cassette-e2e/SKILL.md) como
critério de testes funcionais. A skill REA investiga observações quando
necessário, mas não gera cenários Gherkin ou requisito BDD. A ausência
de HAR/runtime não bloqueia o E2E com respostas gravadas.
