---
type: Contract
title: Diagnóstico do Controle
description: Diagnóstico opt-in do Controle de Processos Web e unidade esperada, parte do contrato da CLI
---

# Diagnóstico do Controle

Parte do [contrato da CLI](../cli.md), separada por responsabilidade.

## Diagnóstico opt-in do Controle de Processos Web (Refs #57)

`ies status --verificar-controle-web [--json]` realiza **um GET**
da URL assinada do Controle de Processos configurada em arquivos
privados, aplicando as restrições existentes de HTTPS, mesma origem,
cookies e ausência de redirects. A operação é explicitamente
solicitada: `ies status` sem opções,
`--verificar-arquivos` e `--verificar-sessao-local`
continuam **sem rede**; opções de diagnóstico são mutuamente
exclusivas. Um GET do SEI pode registrar visualização/recebimento:
usar apenas quando essa leitura estiver autorizada e seus efeitos
forem conhecidos para a instalação.

HTTP 200 isolado, página de login, HTML genérico ou uma página
sem identificação inequívoca da unidade **não são sucesso**.
A resposta precisa conter Controle de Processos reconhecível,
tabelas de Recebidos/Gerados e `lnkInfraUnidade` único.
O diagnóstico não publica números de processos, HTML, cookies,
hashes, URLs nem detalhes de outros usuários, somente
`controle_web: observado`, `unidade_exibida`,
`conexao: get_realizado` e
`autenticacao_remota: nao_auditada`. Falhas (saída 1, stderr): o erro
tipado da configuração ou do transporte e as variantes de
`FalhaControleWeb` (`ControleNaoReconhecido`,
`IdentidadeUnidadeNaoObservada`, `UnidadeExibidaDiferenteDaSolicitada`),
sem classificar mensagens por regex.

Isso não é login automático, teste de permissões para operações
protegidas, prova de cobertura/paginação nem certificação de
autenticação do usuário. Testes live nas instalações autorizadas
seguem exigindo evidência independente; um GET 200 não comprova
o restante da API.

## Unidade esperada no diagnóstico Web (Refs #57, #56)

`ies --unidade SIGLA status --verificar-controle-web --json` confere o
rótulo da unidade **efetivamente publicado** no Controle de Processos
antes de relatar sucesso. A opção global `--unidade` não pode ser
ignorada por este diagnóstico com rede. Em caso de diferença literal,
retornar `UnidadeExibidaDiferenteDaSolicitada` sem imprimir o
rótulo observado nem dados da sessão. Sem `--unidade`, conservar o
resultado atual. A comparação é de igualdade exata, sem heurística
para siglas, normalização ou troca automática de unidade.

A operação continua limitada ao GET já solicitado (zero pedidos
adicionais), sem login ou POST. Este teste de identidade da tela não
comprova sessão autenticada, permissões ou cobertura de paginação.
