---
type: CommandContract
title: Assistente de perfil local
operation: perfil.criar
command: ies wizard [--perfil NOME]
effect: read
implementation: partial
---

# Assistente de perfil local

`ies wizard` cria, por perguntas no terminal, um perfil em
`~/.ies/<perfil>` pronto para `ies login`, com a senha no
[cofre do sistema operacional](credenciais.md). **Sem rede**: nenhuma
requisição ao SEI, nem sondagem, nem login de teste. A conexão fica
`nao_verificada` até o primeiro `ies login`.

## Perguntas

Perguntas em stderr; respostas, uma por linha, na entrada padrão.

1. A instalação: uma das quatro de treinamento com a conta pública que
   ela mesma publica (UnB, PEN1, Goiás e UFRJ, de
   [credenciais públicas](../../docs/interfaces/credenciais-de-treinamento.md))
   ou **outra**. Opção fora da lista é recusa.
2. Só em **outra**: a URL base HTTPS (formato do arquivo de origem), o
   órgão (a sigla da tela de login), o usuário, o nome do perfil (padrão:
   o host) e a senha, pelo terminal **sem eco**, como em
   `ies credenciais salvar`.

`--perfil NOME` escolhe o perfil para qualquer instalação; sem ele, a de
treinamento usa a própria chave (`unb`, `pen1`, `go`, `ufrj`). O perfil
é **um único nome de pasta**: sem `/`, `\`, raiz, prefixo de unidade e
sem começar por `.`. Fora disso, recusa (saída 2 pelo clap com
`--perfil`; `PerfilInvalido` na pergunta), antes de qualquer gravação.

## Efeitos locais, nesta ordem

1. `~/.ies/<perfil>` não pode existir (nem como link): perfil existente é
   `PerfilJaExiste`, sem tocar no cofre nem no disco. Nada é sobrescrito.
2. O cofre é consultado pela chave canônica `(ies:<host>, usuário)`,
   **independente do nome do perfil**. Se a entrada já existe com outra
   senha, recusa `CredencialCompartilhada` antes de escrever; se existe
   com a mesma senha, reutiliza-a sem substituição. Se está ausente,
   salva a senha no cofre. Falhas de acesso ao cofre impedem a escrita no
   disco.
3. Na pasta do perfil (`0700`, criada se faltar, como `~/.ies`),
   cada arquivo é criado exclusivamente com `0600`: `origem`, `cookie`
   e `url-controle` vazios, `credenciais` sem senha e `ies-config.json`.
4. Se qualquer gravação de perfil falhar **depois da criação de uma nova
   entrada do cofre**, o wizard tenta excluir somente essa entrada nova
   e remover a pasta do perfil criado. Uma credencial anteriormente
   existente nunca é excluída nem reescrita. Falha de compensação é
   `RecuperacaoIncompleta`, que conserva o erro original e o motivo do
   cofre/diretório, sem imprimir senhas. Uma pasta que apareça como link
   ou arquivo no momento da compensação não é apagada.

A compensação protege a execução sequencial local, **não é uma
transação distribuída ou trava interprocessos**: alterações concorrentes
no mesmo cofre e pasta permanecem fora desta garantia. Em
`RecuperacaoIncompleta`, inspecionar o estado local antes de repetir;
não repetir automaticamente a criação de perfil.

## Saída

```json
{"perfil":"unb","cofre":"ies:sei.treinamento.unb.br",
 "config_file":"/home/p/.ies/unb/ies-config.json",
 "credenciais_file":"/home/p/.ies/unb/credenciais",
 "conexao":"nao_verificada"}
```

Sem `--json`, as mesmas informações e as linhas a copiar antes de
`ies login`: `export IES_CONFIG_FILE=…` e `export IES_CREDENCIAIS_FILE=…`
(sh, bash, zsh) e `$env:IES_CONFIG_FILE = …` e
`$env:IES_CREDENCIAIS_FILE = …` (PowerShell), cada caminho entre aspas
simples com o escape do próprio shell: espaço, `$`, crase e aspas no
caminho continuam dado, nunca comando. Nenhum script é gravado. Nunca a
senha.

## Falhas tipadas

Saída 1 com `Error: ErroWizard::…` em stderr: `PastaPessoalDesconhecida`,
`EntradaEncerrada`, `EntradaIlegivel`, `OpcaoInvalida`,
`OrigemNaoReconhecida`, `OrgaoVazio`, `UsuarioVazio`, `PerfilInvalido`,
`PerfilJaExiste`, `PerfilNaoConsultado`, `CredencialCompartilhada`,
`RecuperacaoIncompleta`, `Senha(ErroCredenciais)`,
`Cofre(ErroCofre)` ([cofre](credenciais.md#falhas-tipadas)),
`Gravacao(FalhaDaGravacao)` e
`Configuracao(ErroGravacaoDaConfiguracao)`. Sem cofre utilizável
(servidor, CI), o assistente falha sem gravar nada; o perfil se monta à
mão com `senha=` no arquivo `0600` e `ies setup`.
