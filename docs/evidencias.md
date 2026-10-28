---
type: Guide
title: Evidências por operação
description: Como relacionar o contrato da CLI às cassettes verificáveis sem falsas provas
---

# Operações do Todos e evidências do SEI

O [índice OKF](../contracts/evidencias/README.md) reúne as **129 operações**
presentes na matriz do Todos no commit `3d4a38c3b5987e4a8fc7f97ded188c4876ed77cd`.

É um **inventário de evidência**, não uma decisão de que todas as operações
sejam contratos públicos, tampouco de que estejam implementadas no Rust.
A definição da CLI do IES continua em [contratos](../contracts/cli.md).

## Para implementar uma operação

1. Consulte o [card da operação](../contracts/evidencias/README.md), por exemplo
   [processo.enviar](../contracts/evidencias/processo-enviar.md).
2. Leia o estado atribuído pelo Todos para cada instalação: `demonstrada`,
   `na` (não aplicável) ou `pendente`.
3. Siga o link do cenário e o número do passo (`cenario#passo`) até o
   [card da cassette](cassettes.md), e deste até o ZIP original no Internet Archive.
4. Confira os hashes e compare a **revisão específica** com o manifesto atual
   do Todos antes de usar a gravação como evidência do passo.
5. Faça replay estrutural offline e escreva testes do contrato antes
   de implementar a operação HTTP no IES.

## Cobertura atualizada dos ZIPs

A atualização de 29 ZIPs no Internet Archive acrescenta 17 revisões
distintas, levando o catálogo a 495 cards. A matriz permanece com
129 operações e 2.154 referências de passos. A atualização não modifica
o contrato de execução no IES.

## Lacunas conhecidas

A matriz registra **2.154 referências de passos**. As **76 referências anteriormente ausentes** da família `repo` agora
apontam para cassettes de novos lotes publicados. Permanecem **zero referências
sem card**. Quando existem duas revisões do mesmo cenário, ambas ficam
visíveis, sem escolha arbitrária. A correspondência exata de revisão
ainda depende da conferência com o manifesto do Todos.

Mesmo as relações com card encontrado não provam ainda qual
**revisão exata** foi usada na demonstração da matriz. Revisões
duplicadas por cenário permanecem distintas, não se seleciona a primeira.

As gravações não incluem todos os corpos das requisições POST. A
matriz do Todos registra demonstrações, mas **o IES não as herda como
certificação automática de sua implementação**.

## Gates

`cargo run --locked --bin check_evidence` valida que os cards
possuem proveniência e que todos os links relativos resolvem
para conceitos OKF `Cassette` com instalação correspondente.
O `prek` e o GitHub Actions executam o mesmo verificador.
