---
type: Contract
title: Contrato da CLI IES
description: Regras publicas provisórias da interface de linha de comando
---

# Contrato CLI do IES — fundação provisória

A interface CLI é a especificação primária; a implementação será guiada pelos
testes e pelas instalações reais.

## Invariantes

- Interface exclusivamente CLI, em português, orientada por substantivos e verbos de domínio.
- `ies <substantivo> NUMERO` lê; `ies <substantivo>` sem argumentos mostra ajuda.
- Não existe `agir`, `Action` ou `PageModel` como contrato público.
- `--json` e `--unidade` são globais; falhas operacionais são estruturadas.
- Códigos de saída: 0 sucesso, 1 falha, 2 uso inválido, 3 efeito incerto.
- Escritas exigem prévia, confirmação e verificação; efeito incerto nunca se repete automaticamente.
- Comportamentos são mapeados empiricamente em instalações reais do SEI.
- Esta superfície de demonstração não é integral: o inventário será aprovado comando a comando.

A [matriz de evidências](evidencias/README.md) relaciona operações
registradas pelo Todos a cassettes, mas **não equivale a aprovação dos
contratos desta CLI**. Contrato e evidência são fontes distintas.

## Independência de implementação

O contrato descreve somente comportamento e efeitos observáveis.
A decisão de usar REST, SOAP ou HTML pertence a um adaptador específico
e precisa de [evidência comparativa](../docs/arquitetura/0001-rotas-por-evidencia.md).
Nenhum método HTTP, resultado de uma API não habilitada ou mecanismo
legado do Todos faz parte do contrato público por pressuposição.

## Primeiro recorte, com acesso Web estritamente limitado

- `ies processo NUP`
- `ies documento NUMERO`
- `ies unidade listar-processos`
- `ies processo enviar NUP --para SIGLA`
- `ies documento assinar NUMERO`
- `ies status` (opção `--verificar-arquivos` confere metadados locais sem rede)
- `ies setup`
- `ies esquema`

## Contratos iniciais de leitura

- [processo.obter](cli/processo-obter.md): leitura do processo e projeções.
- [documento.obter](cli/documento-obter.md): conteúdo de um ou mais documentos.
- [unidade.listar_processos](cli/unidade-listar-processos.md): caixa de processos.

O comando `ies unidade listar-processos` possui **leitura HTTP GET
restrita da caixa Web**, com sessão já existente e origem HTTPS
explicitamente autorizada, ou replay offline verificado. Não faz
login, POST ou paginação HTTP. Sem esses insumos retorna
`nao_implementado`.

`processo NUP`, `documento NUMERO` e comandos de escrita continuam
como stubs. A autorização e a equivalência de outras rotas exigem
ensaios por cassette e por instalação.
