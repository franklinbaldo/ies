---
type: Guide
title: Diagnóstico seguro dos arquivos locais do IES
description: Contrato da verificação opt-in de metadados, sem conexão ao SEI
---

# Status da configuração local

O comando `ies status` conserva a consulta de caminhos informados e não
abre os três arquivos privados nem envia requisições ao SEI.

`ies status --verificar-arquivos --json` verifica **somente metadados**
dos caminhos da URL de controle, cookie e origem autorizada, nesta ordem.
Exige caminho absoluto, arquivo existente, regular, não vazio, não simbólico
e, em Unix, sem permissão de grupo/outros (modo equivalente a 0600 ou
mais restrito). Não lê conteúdo, não verifica autenticidade de cookie,
não interpreta URL e nunca prova conexão ou disponibilidade de SEI.

O sucesso retorna `arquivos_privados: metadados_verificados` com
`conexao: nao_verificada` e código 0. Falhas retornam código 1,
`codigo: configuracao_invalida`, `subcodigo` estável e
`arquivo` com um dos papéis `url_controle`, `cookie`,
`origem`, sem expor caminhos nem conteúdos. Variantes tipadas:
`CaminhoNaoAbsoluto`, `Ausente`, `LinkSimbolico`,
`NaoRegular`, `Vazio`, `PermissoesInseguras` e `MetadadosNaoLidos`.
A ausência de configuração deve falhar antes da inspeção dos arquivos.
Sem `--verificar-arquivos`, a saída existente não muda.

Comportamento local; testes sintéticos bastam para este escopo.
Validação live, gravação de cassettes e publicação: NOT_RUN por não
fazerem parte de uma operação de configuração local.
