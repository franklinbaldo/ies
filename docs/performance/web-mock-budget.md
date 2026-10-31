---
type: Guide
title: Orçamento de desempenho do transporte Web simulado
description: Medição reproduzível e limites de comparação da primeira leitura HTTP do IES
---

# Transporte Web: orçamento de requisições e desempenho offline

A issue [#95](https://github.com/franklinbaldo/ies/issues/95) distingue a
velocidade do parser local da latência e do número de requisições Web.

## Contrato do experimento

A unidade solicita **um GET** para a URL já autorizada da caixa. O transporte
confere origem, URL assinada e arquivo de cookie privado **antes de chamar o
adaptador HTTP**; depois aceita somente HTTP 200 HTML, sem redirecionamento,
limita o corpo recebido a 8 MB e decodifica a resposta. O mesmo HTML chega
ao parser de caixa e à projeção de saída, sem segundo GET. A operação não
executa login, POST ou escrita no SEI.

Testes de contrato devem contar a invocação efetiva do adaptador, inclusive
nos casos negativos (origem inválida deve resultar em **zero** GET), além de
verificar erro tipado, cobertura das tabelas e projeção de saída sem URLs
assinadas. Duplicar respostas sintéticas e inventar página adicional não
é permitido: páginas não observadas não devem ser marcadas completas.

## Medição

O benchmark em Rust executa o transporte injetado, a verificação do corpo
HTTP, o parser da caixa e a serialização do resultado, tudo sobre HTML
sintético em memória. Reporta número de invocações e bytes, p50/p95 de
latência para 20 e 1.000 processos. Tempo não é critério de segurança,
nem representa RTT, TLS, DNS, autenticação ou desempenho de uma instalação.

As execuções de performance usam o próprio pipeline da CLI; não desabilitam
as validações nem alteram a política de autorização. O teste de orçamento
asserta **exatamente um** GET simulado por operação válida, e zero em
pré-condições recusadas. Percentis são informativos, nunca motivos
isolados para rejeitar um PR em máquinas CI de carga variável.

## Limitações e próximo experimento

São testes sem rede. O transporte live continua restrito a GET da primeira
tela; sem sessão de treinamento autorizada não há validação live de efeitos.
A comparação completa com o Todos exigirá o mesmo HTML, funcionalidade
equivalente, mesmo executor e contagem de HTTP por operação, sem mover
código ou credenciais do repositório privado para o CI público.
