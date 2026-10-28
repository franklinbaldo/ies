---
type: Guide
title: Mudanças recentes do Todos relevantes ao IES
description: Lições de compatibilidade para próximas operações sem portar o código Python
---

# Regressões e decisões úteis do Todos

Registro documental revisado em 28/10/2026. O Todos fornece
casos de teste e descobertas, **não classes, protocolos ou arquitetura**
para copiar no IES. Distinguir implementado na main, issue aberta
e PR empilhada em outra branch.

## Caixa, contexto e paginação

A [issue #1200](https://github.com/franklinbaldo/todos/issues/1200)
relata, na unidade PGE-IPERON em Rondônia, que a **visão resumida
também pagina** acima de 500 processos. Foram 508 itens retornados
quando o relatório de acervo indicava 769. A tela usa controles
independentes `hdnRecebidosPaginaAtual` e
`hdnGeradosPaginaAtual`; não assumir que "resumida" = única página.
O relato menciona trocar para a visão detalhada e restaurar a
preferência depois, mas isso **ainda não autoriza o IES a emitir
POST ou alterar estado de sessão**.

Para o IES, preservar a prova de faixas e as duas tabelas separadas.
Agora também comparar o rótulo da unidade e os critérios das páginas;
ausência de evidência impede mescla. Contexto coincidente **não é
snapshot transacional**.

## Identidade e links de leitura

A [PR #1194](https://github.com/franklinbaldo/todos/pull/1194),
integrada à main do Todos em 26/10/2026, removeu referências
`sei://` e passou a usar número exibido e ID interno do SEI,
além de URL estável sem `infra_hash`. O relato descreve
ensaios na UnB, Goiás, PEN3 e PEN4. Isso reforça a separação de
`IdProcedimento` e NUP no IES, mas **não autoriza construir
URLs arbitrárias ou tratar um link estável como acesso autorizado**.

## Árvores e pastas parcialmente carregadas

A [PR #1228](https://github.com/franklinbaldo/todos/pull/1228)
foi integrada à branch da [PR #1226](https://github.com/franklinbaldo/todos/pull/1226),
que chegou à main em 27/10/2026. Relata que uma pasta
parcialmente carregada não deve transformar seus nós em
documentos íntegros, nem necessariamente bloquear **um único
documento inequívoco de outra pasta verificada**.

Quando o IES implementar `documento NUMERO` e a árvore de
`processo NUP`, criar testes de pastas incompletas e de
escopo único/lote; não implementar recuperação com POST antes
de evidência específica do formulário.

## Edição HTML e finais de linha

A [PR #1226](https://github.com/franklinbaldo/todos/pull/1226),
na main, trata falsos conflitos causados por serialização de HTML
(CRLF/LF, `<br>` e espaços entre elementos), sem ignorar
edição concorrente real.

A [PR #1235](https://github.com/franklinbaldo/todos/pull/1235)
foi mergeada em 28/10/2026 **em uma branch empilhada,
não na main**, com correção adicional de CR solto/CRLF para
gerar cópia de trabalho e base em LF, gravadas como bytes UTF-8.
Quando houver `documento baixar/salvar`, preservar fidelidade
da origem e controlar explicitamente normalização das **cópias
editáveis e de suas bases**, não reescrever HTML recebido na
leitura do SEI por conveniência.

## Números SEI, marcadores vazios

A [issue #1230](https://github.com/franklinbaldo/todos/issues/1230)
mostra ambiguidades com zeros à esquerda na pesquisa de números
SEI; a [PR #1232](https://github.com/franklinbaldo/todos/pull/1232)
foi fechada **sem merge**. Não pressupor largura fixa ou
normalização unilateral: reproduzir nas instalações e escolher
entre pesquisa rápida ou diagnóstico explícito de formatos.

A [issue #1233](https://github.com/franklinbaldo/todos/issues/1233)
registra que marcador existente sem processos pode desaparecer do
filtro de caixa. Ao implementar marcadores, não confundir resultado
vazio com entidade inexistente; cotejar o catálogo da unidade.

## Ordem prática

Primeiro corrigir o transporte HTTPS e conferir unidade/filtro/modo
na caixa Web. Depois implementar sessão e paginação autorizadas.
Quando `processo` e `documento` forem implementados, acrescentar
os testes discriminantes acima. **Nada disso obriga a reproduzir
o desenho interno do Todos.**
